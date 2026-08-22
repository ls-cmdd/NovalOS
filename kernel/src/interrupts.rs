use core::arch::asm;
use core::arch::global_asm;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

global_asm!(
    r#"
    .global syscall_entry
    .type syscall_entry, @function
syscall_entry:
    movq %rsp, %r8
    movq %rdx, %rcx
    movq %rdi, %r10
    movq %rsi, %r11
    movq %rax, %rdi
    movq %r10, %rsi
    movq %r11, %rdx
    callq syscall_dispatch
    iretq
    "#,
    options(att_syntax)
);

use spin::{Mutex, Once};
use x86_64::registers::control::Cr2;
use x86_64::{
    PrivilegeLevel, VirtAddr,
    structures::idt::{
        InterruptDescriptorTable, InterruptStackFrame, InterruptStackFrameValue, PageFaultErrorCode,
    },
};

use crate::{fs, gdt, keyboard, memory, pic, syscall, task, timer, user};

unsafe extern "C" {
    fn syscall_entry();
    fn timer_entry();
}

static IDT: Once<InterruptDescriptorTable> = Once::new();
static SYSCALL_TABLE: syscall::SyscallTable = syscall::SyscallTable::new();
static EXIT_STATUS: AtomicU64 = AtomicU64::new(0);

fn build_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    // SAFETY: index 0 is backed by the dedicated stack configured in the TSS,
    // and no other interrupt uses this IST slot.
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.page_fault.set_handler_fn(page_fault_handler);
    // SAFETY: syscall_entry is a well-formed interrupt-gate ABI wrapper;
    // it consumes the CPU-pushed frame with iretq after calling Rust.
    unsafe {
        idt[0x80]
            .set_handler_addr(VirtAddr::new(syscall_entry as *const () as usize as u64))
            .set_privilege_level(PrivilegeLevel::Ring3);
    }
    // IRQ0 uses the assembly wrapper because it must preserve the complete
    // GP-register set before Rust touches the CPU-pushed user frame.
    unsafe {
        idt[pic::TIMER_VECTOR as u8]
            .set_handler_addr(VirtAddr::new(timer_entry as *const () as usize as u64));
    }
    idt[pic::KEYBOARD_VECTOR as u8].set_handler_fn(keyboard_interrupt_handler);
    idt
}

pub fn init() {
    let idt = IDT.call_once(build_idt);

    // The IDT is stored in a Once cell and therefore remains at a stable
    // address for the rest of the kernel lifetime, as required by lidt.
    idt.load();
}

/// Build the shared immutable interrupt table in a low-memory frame on the BSP.
///
/// Building is deliberately done before AP entry. It resolves all handler
/// addresses while the BSP's complete kernel mapping is active; the AP only
/// performs `lidt` on the finished frame.
pub unsafe fn build_idt_at(address: VirtAddr) -> bool {
    if core::mem::size_of::<InterruptDescriptorTable>() > 4096 || address.as_u64() & 0xfff != 0 {
        return false;
    }
    let table = address.as_mut_ptr::<InterruptDescriptorTable>();
    // SAFETY: caller supplies an allocated, identity-mapped, writable 4 KiB frame.
    unsafe { table.write(build_idt()) };
    true
}

/// Load a previously built low-memory IDT on an AP.
///
/// No high-half Rust data is touched here: the table is complete and immutable,
/// and the AP's own GDT/TSS have already been loaded.
pub unsafe extern "C" fn init_ap_at(address: VirtAddr) -> bool {
    if core::mem::size_of::<InterruptDescriptorTable>() > 4096 || address.as_u64() & 0xfff != 0 {
        return false;
    }
    let table = address.as_ptr::<InterruptDescriptorTable>();
    // SAFETY: the frame is permanent and was initialized by build_idt_at.
    let table: &'static InterruptDescriptorTable = unsafe { &*table };
    table.load();
    true
}

/// Deliberately execute INT3 to prove that the CPU is using our IDT.
pub fn test_breakpoint() {
    unsafe {
        asm!("int3", options(nomem, nostack, preserves_flags));
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    keyboard::on_interrupt();
}

#[unsafe(no_mangle)]
pub extern "C" fn syscall_dispatch(
    number: u64,
    pointer: u64,
    length: u64,
    extra: u64,
    frame: *const InterruptStackFrameValue,
) -> u64 {
    // The assembly wrapper passes the CPU-owned frame pointer before calling
    // Rust. Reading it is safe because the wrapper does not alter that frame.
    let cpl3 = unsafe { (*frame).code_segment.rpl() == PrivilegeLevel::Ring3 };
    let mut serial = crate::SerialWriter::new();
    let _ = writeln!(serial, "RING3 int80: cpl={} valid={cpl3}", unsafe {
        (*frame).code_segment.rpl() as u8
    });
    if !cpl3 {
        let _ = writeln!(serial, "SYSCALL rejected: not CPL3");
        return syscall::Error::PermissionDenied as u64;
    }

    match number {
        n if n == syscall::Number::Write as u64 => {
            let pointer_ok = user::validate_user_read(pointer, length) && length <= 4096;
            if !pointer_ok {
                let _ = writeln!(serial, "SYSCALL write: rejected invalid_user_pointer");
                return syscall::Error::InvalidArgument as u64;
            }
            let bytes =
                unsafe { core::slice::from_raw_parts(pointer as *const u8, length as usize) };
            for &byte in bytes {
                let _ = serial.write_char(byte as char);
            }
            let _ = writeln!(serial, "SYSCALL write: accepted length={length}");
            length
        }
        n if n == syscall::Number::Open as u64 => {
            if length == 0 || length > 64 || !user::validate_user_read(pointer, length) {
                let _ = writeln!(serial, "SYSCALL open: rejected invalid_name_pointer");
                return syscall::Error::InvalidArgument as u64;
            }
            let mut name = [0_u8; 64];
            unsafe {
                core::ptr::copy_nonoverlapping(
                    pointer as *const u8,
                    name.as_mut_ptr(),
                    length as usize,
                );
            }
            let Some(rights) = fs::Rights::from_bits(extra) else {
                let _ = writeln!(serial, "SYSCALL open: rejected invalid_rights");
                return syscall::Error::PermissionDenied as u64;
            };
            match fs::global_open(
                &name[..length as usize],
                rights,
                task::current_user_process_id(),
            ) {
                Ok(handle) => {
                    let raw = handle.raw();
                    let _ = writeln!(serial, "SYSCALL open: handle={raw}");
                    raw
                }
                Err(error) => {
                    let _ = writeln!(serial, "SYSCALL open: error={error:?}");
                    fs_error_code(error)
                }
            }
        }
        n if n == syscall::Number::Read as u64 => {
            if !user::validate_user_write(pointer, length) || length > 4096 {
                let _ = writeln!(serial, "SYSCALL read: rejected invalid_user_buffer");
                return syscall::Error::InvalidArgument as u64;
            }
            let Some(handle) = fs::FileHandle::from_raw(extra) else {
                let _ = writeln!(serial, "SYSCALL read: rejected invalid_handle");
                return syscall::Error::InvalidHandle as u64;
            };
            let output =
                unsafe { core::slice::from_raw_parts_mut(pointer as *mut u8, length as usize) };
            match fs::global_read(handle, output, task::current_user_process_id()) {
                Ok(count) => {
                    let _ = writeln!(
                        serial,
                        "SYSCALL read: handle={} bytes={count}",
                        handle.raw()
                    );
                    count as u64
                }
                Err(error) => {
                    let _ = writeln!(serial, "SYSCALL read: error={error:?}");
                    fs_error_code(error)
                }
            }
        }
        n if n == syscall::Number::FileWrite as u64 => {
            if !user::validate_user_read(pointer, length) || length > 4096 {
                let _ = writeln!(serial, "SYSCALL file_write: rejected invalid_user_buffer");
                return syscall::Error::InvalidArgument as u64;
            }
            let Some(handle) = fs::FileHandle::from_raw(extra) else {
                let _ = writeln!(serial, "SYSCALL file_write: rejected invalid_handle");
                return syscall::Error::InvalidHandle as u64;
            };
            let input =
                unsafe { core::slice::from_raw_parts(pointer as *const u8, length as usize) };
            match fs::global_write(handle, input, task::current_user_process_id()) {
                Ok(count) => {
                    let _ = writeln!(
                        serial,
                        "SYSCALL file_write: handle={} bytes={count}",
                        handle.raw()
                    );
                    count as u64
                }
                Err(error) => {
                    let _ = writeln!(serial, "SYSCALL file_write: error={error:?}");
                    fs_error_code(error)
                }
            }
        }
        n if n == syscall::Number::Exit as u64 => {
            let status = pointer;
            task::mark_current_user_process_exited(status, false);
            EXIT_STATUS.store(status, Ordering::Release);
            let _ = writeln!(serial, "SYSCALL exit: handoff status={status}");
            redirect_to_process_reaper(frame);
            0
        }
        n if n == syscall::Number::Close as u64 => {
            let Some(handle) = fs::FileHandle::from_raw(pointer) else {
                let _ = writeln!(serial, "SYSCALL close: rejected invalid_handle");
                return syscall::Error::InvalidHandle as u64;
            };
            match fs::global_close(handle, task::current_user_process_id()) {
                Ok(()) => {
                    let _ = writeln!(serial, "SYSCALL close: handle={} closed=true", handle.raw());
                    0
                }
                Err(error) => {
                    let _ = writeln!(serial, "SYSCALL close: error={error:?}");
                    fs_error_code(error)
                }
            }
        }
        n if n == syscall::Number::PortCreate as u64 => {
            match crate::ipc::create(task::current_user_process_id()) {
                Ok(handle) => {
                    let _ = writeln!(serial, "SYSCALL port_create: handle={handle}");
                    handle
                }
                Err(error) => {
                    let _ = writeln!(serial, "SYSCALL port_create: error={error:?}");
                    ipc_error_code(error)
                }
            }
        }
        n if n == syscall::Number::PortSend as u64 => {
            if length > 64 || !user::validate_user_read(pointer, length) {
                let _ = writeln!(serial, "SYSCALL port_send: rejected invalid_message");
                return syscall::Error::InvalidArgument as u64;
            }
            let input =
                unsafe { core::slice::from_raw_parts(pointer as *const u8, length as usize) };
            match crate::ipc::send(extra, task::current_user_process_id(), input) {
                Ok(count) => {
                    let _ = writeln!(serial, "SYSCALL port_send: handle={extra} bytes={count}");
                    count as u64
                }
                Err(error) => {
                    let _ = writeln!(serial, "SYSCALL port_send: error={error:?}");
                    ipc_error_code(error)
                }
            }
        }
        n if n == syscall::Number::PortRecv as u64 => {
            if length > 64 || !user::validate_user_write(pointer, length) {
                let _ = writeln!(serial, "SYSCALL port_recv: rejected invalid_message");
                return syscall::Error::InvalidArgument as u64;
            }
            let output =
                unsafe { core::slice::from_raw_parts_mut(pointer as *mut u8, length as usize) };
            match crate::ipc::recv(extra, task::current_user_process_id(), output) {
                Ok(count) => {
                    let _ = writeln!(serial, "SYSCALL port_recv: handle={extra} bytes={count}");
                    count as u64
                }
                Err(error) => {
                    let _ = writeln!(serial, "SYSCALL port_recv: error={error:?}");
                    ipc_error_code(error)
                }
            }
        }
        _ => match SYSCALL_TABLE.dispatch(number, [pointer, length, extra]) {
            Ok(syscall::Return::Value(value)) => {
                let _ = writeln!(serial, "SYSCALL dispatch: number={number} return={value}");
                value
            }
            Ok(syscall::Return::Exited(status)) => {
                let _ = writeln!(serial, "SYSCALL dispatch: exit status={status}");
                status
            }
            Err(error) => {
                let _ = writeln!(serial, "SYSCALL dispatch: number={number} error={error:?}");
                error as u64
            }
        },
    }
}

fn redirect_to_process_reaper(frame: *const InterruptStackFrameValue) {
    let selectors = gdt::user_selectors();
    let frame = unsafe { &mut *(frame as *mut InterruptStackFrameValue) };
    frame.instruction_pointer = VirtAddr::new(process_exit_reaper as *const () as usize as u64);
    frame.code_segment = selectors.code;
    frame.stack_pointer = gdt::kernel_stack_top();
    frame.stack_segment = selectors.data;
}

#[unsafe(no_mangle)]
extern "C" fn process_exit_reaper() -> ! {
    let status = EXIT_STATUS.load(Ordering::Acquire);
    let owner = task::current_user_process_id();
    let recorded_status = task::user_process_exit_status(owner).unwrap_or(status);
    let state = task::user_process_state(owner).unwrap_or(task::UserProcessState::Empty);
    let revoked_handles = fs::global_revoke_owner(owner);
    let revoked_ports = crate::ipc::revoke_owner(owner);
    remove_user_context(owner);
    let reclaimed = unsafe { memory::reap_active_process() };
    let next_process = unsafe { memory::next_process_after_reap(owner) };
    let mut serial = crate::SerialWriter::new();
    let _ = writeln!(
        serial,
        "user preemption: switches={} frames=true starvation=false",
        user_preemption_switches()
    );
    let _ = writeln!(
        serial,
        "PROCESS EXIT: status={recorded_status} owner={owner} state={state:?} revoked_handles={revoked_handles} revoked_ports={revoked_ports} reclaimed={reclaimed} next={}",
        next_process.is_some()
    );
    let Some((next_id, _next_process)) = next_process else {
        USER_QUEUE_IDLE.store(true, Ordering::Release);
        let _ = writeln!(serial, "USER QUEUE: idle=true runnable=0");
        halt_forever();
    };
    if !task::set_current_user_process(next_id) {
        halt_forever();
    }
    if !unsafe { memory::switch_to_process(next_id) }
        || !select_user_context(next_id)
        || !gdt::set_process_kernel_stack(next_id)
    {
        halt_forever();
    }
    let Some(saved_context) = saved_user_context(next_id) else {
        halt_forever();
    };
    let _ = writeln!(
        serial,
        "PROCESS HANDOFF: from={owner} to={next_id} state=Running cr3=switched active={} resume=saved",
        memory::active_process_present()
    );
    unsafe { resume_user_context(&saved_context) }
}

fn ipc_error_code(error: crate::ipc::IpcError) -> u64 {
    match error {
        crate::ipc::IpcError::InvalidHandle => syscall::Error::InvalidHandle as u64,
        crate::ipc::IpcError::PermissionDenied => syscall::Error::PermissionDenied as u64,
        crate::ipc::IpcError::QueueFull
        | crate::ipc::IpcError::QueueEmpty
        | crate::ipc::IpcError::InvalidMessage => syscall::Error::InvalidArgument as u64,
    }
}

fn fs_error_code(error: fs::FsError) -> u64 {
    match error {
        fs::FsError::InvalidHandle => syscall::Error::InvalidHandle as u64,
        fs::FsError::NotFound => syscall::Error::NotFound as u64,
        fs::FsError::PermissionDenied => syscall::Error::PermissionDenied as u64,
        _ => syscall::Error::InvalidArgument as u64,
    }
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    let mut serial = crate::SerialWriter::new();
    let _ = writeln!(serial, "EXCEPTION: BREAKPOINT (#BP) handled");
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    let mut serial = crate::SerialWriter::new();
    let _ = writeln!(serial, "EXCEPTION: DOUBLE FAULT (#DF), error={error_code}");
    halt_forever();
}

extern "x86-interrupt" fn page_fault_handler(
    mut stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let address = Cr2::read();
    let from_user = stack_frame.code_segment.rpl() == PrivilegeLevel::Ring3;
    let mut serial = crate::SerialWriter::new();
    if from_user {
        // A user fault is terminal for this process. Encode the architectural
        // error bits in a reserved status range and reuse the same kernel
        // reaper path as syscall Exit; never guess an instruction length.
        let status = 0xF000_u64 | (error_code.bits() as u64 & 0x0FFF);
        task::mark_current_user_process_exited(status, true);
        EXIT_STATUS.store(status, Ordering::Release);
        let _ = writeln!(
            serial,
            "USER FAULT POLICY: kill=true status={status} address={address:?} error={error_code:?}"
        );
        let mut frame = unsafe { stack_frame.as_mut() };
        let mut value = frame.read();
        let selectors = gdt::user_selectors();
        value.instruction_pointer = VirtAddr::new(process_exit_reaper as *const () as usize as u64);
        value.code_segment = selectors.code;
        value.stack_pointer = gdt::kernel_stack_top();
        value.stack_segment = selectors.data;
        frame.write(value);
        return;
    }
    let _ = writeln!(
        serial,
        "EXCEPTION: PAGE FAULT (#PF), address={address:?}, error={error_code:?}"
    );
    halt_forever();
}

fn halt_forever() -> ! {
    loop {
        unsafe {
            asm!("cli; hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

global_asm!(
    r#"
    .global resume_user_context
    .type resume_user_context, @function
resume_user_context:
    # SavedUserContext = 15 GP registers (120 bytes) + InterruptStackFrameValue (40 bytes).
    # Push the architectural iret frame before restoring registers; iretq consumes it.
    pushq 152(%rdi)          # stack segment
    pushq 144(%rdi)          # user RSP
    pushq 136(%rdi)          # RFLAGS
    pushq 128(%rdi)          # code segment
    pushq 120(%rdi)          # instruction pointer
    movq 0(%rdi), %r15
    movq 8(%rdi), %r14
    movq 16(%rdi), %r13
    movq 24(%rdi), %r12
    movq 32(%rdi), %r11
    movq 40(%rdi), %r10
    movq 48(%rdi), %r9
    movq 56(%rdi), %r8
    movq 64(%rdi), %rbp
    # Keep the context pointer in %rdi until every later field is loaded.
    movq 80(%rdi), %rsi
    movq 88(%rdi), %rdx
    movq 96(%rdi), %rcx
    movq 104(%rdi), %rbx
    movq 112(%rdi), %rax
    movq 72(%rdi), %rdi
    iretq
    "#,
    options(att_syntax)
);

unsafe extern "C" {
    fn resume_user_context(context: *const SavedUserContext) -> !;
}

global_asm!(
    r#"
    .global timer_entry
    .type timer_entry, @function
timer_entry:
    pushq %rax
    pushq %rbx
    pushq %rcx
    pushq %rdx
    pushq %rsi
    pushq %rdi
    pushq %rbp
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    movq %rsp, %rdi
    callq timer_dispatch
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %r11
    popq %r10
    popq %r9
    popq %r8
    popq %rbp
    popq %rdi
    popq %rsi
    popq %rdx
    popq %rcx
    popq %rbx
    popq %rax
    iretq
    "#,
    options(att_syntax)
);

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct SavedGpRegisters {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SavedUserContext {
    registers: SavedGpRegisters,
    frame: InterruptStackFrameValue,
}

static USER_PREEMPTION_READY: AtomicBool = AtomicBool::new(false);
static USER_PREEMPT_SWITCHES: AtomicU64 = AtomicU64::new(0);
static USER_QUEUE_IDLE: AtomicBool = AtomicBool::new(false);
const USER_CONTEXT_CAPACITY: usize = 3;
struct UserContextRegistry {
    contexts: [Option<SavedUserContext>; USER_CONTEXT_CAPACITY],
    ids: [u64; USER_CONTEXT_CAPACITY],
    current: usize,
}

impl UserContextRegistry {
    const fn new() -> Self {
        Self {
            contexts: [None; USER_CONTEXT_CAPACITY],
            ids: [0; USER_CONTEXT_CAPACITY],
            current: 0,
        }
    }
}

static USER_CONTEXT_REGISTRY: Mutex<UserContextRegistry> = Mutex::new(UserContextRegistry::new());

#[unsafe(no_mangle)]
extern "C" fn timer_dispatch(saved_registers: *mut SavedGpRegisters) {
    let _interrupt_guard = crate::smp::interrupt_guard();
    timer::on_interrupt();
    if crate::smp::running_on_ap() {
        // AP timer interrupts are acknowledged but cannot touch the BSP-owned
        // process/scheduler context table until that table has an SMP-safe
        // ownership protocol. Preserve the event in the AP-local record.
        crate::smp::defer_work(1);
        return;
    }
    if !USER_PREEMPTION_READY.load(Ordering::Acquire) {
        task::on_timer_tick();
        return;
    }
    let frame_pointer = unsafe {
        (saved_registers as *mut u8).add(core::mem::size_of::<SavedGpRegisters>())
            as *mut InterruptStackFrameValue
    };
    let frame = unsafe { core::ptr::read_volatile(frame_pointer) };
    if frame.code_segment.rpl() != PrivilegeLevel::Ring3 {
        task::on_timer_tick();
        return;
    }
    let mut registry = USER_CONTEXT_REGISTRY.lock();
    let current = registry.current;
    let Some((next, current_id, next_id)) = find_next_runnable_context(&registry, current) else {
        USER_QUEUE_IDLE.store(false, Ordering::Release);
        return;
    };
    if current_id == 0 || next_id == 0 || current_id == next_id {
        return;
    }
    let current_context = SavedUserContext {
        registers: unsafe { core::ptr::read(saved_registers) },
        frame,
    };
    let Some(next_context) = registry.contexts[next] else {
        return;
    };
    if !task::switch_current_user_process(next_id) {
        return;
    }
    registry.contexts[current] = Some(current_context);
    registry.current = next;
    // SAFETY: next_id came from the locked context registry and its process
    // root was validated by memory::switch_to_process before CR3 mutation.
    let _ = unsafe { memory::switch_to_process(next_id) };
    let _ = gdt::set_process_kernel_stack(next_id);
    unsafe {
        core::ptr::write(saved_registers, next_context.registers);
        core::ptr::write_volatile(frame_pointer, next_context.frame);
    }
    USER_PREEMPT_SWITCHES.fetch_add(1, Ordering::Relaxed);
    drop(registry);
}

fn find_next_runnable_context(
    registry: &UserContextRegistry,
    current: usize,
) -> Option<(usize, u64, u64)> {
    for offset in 1..=USER_CONTEXT_CAPACITY {
        let index = (current + offset) % USER_CONTEXT_CAPACITY;
        let context_present = registry.contexts[index].is_some();
        let current_id = registry.ids[current];
        let next_id = registry.ids[index];
        if !context_present || next_id == 0 || current_id == 0 || current_id == next_id {
            continue;
        }
        let next_runnable = matches!(
            task::user_process_state(next_id),
            Some(task::UserProcessState::Ready | task::UserProcessState::Running)
        );
        if next_runnable {
            return Some((index, current_id, next_id));
        }
    }
    None
}

pub unsafe fn init_user_preemption(
    first_id: u64,
    second_id: u64,
    third_id: u64,
    first_entry: u64,
    first_stack: u64,
    second_entry: u64,
    second_stack: u64,
    third_entry: u64,
    third_stack: u64,
) {
    let selectors = gdt::user_selectors();
    let flags = x86_64::registers::rflags::RFlags::INTERRUPT_FLAG;
    let first_frame = InterruptStackFrameValue::new(
        VirtAddr::new(first_entry),
        selectors.user_code,
        flags,
        VirtAddr::new(first_stack),
        selectors.user_data,
    );
    let second_frame = InterruptStackFrameValue::new(
        VirtAddr::new(second_entry),
        selectors.user_code,
        flags,
        VirtAddr::new(second_stack),
        selectors.user_data,
    );
    let third_frame = InterruptStackFrameValue::new(
        VirtAddr::new(third_entry),
        selectors.user_code,
        flags,
        VirtAddr::new(third_stack),
        selectors.user_data,
    );
    USER_PREEMPT_SWITCHES.store(0, Ordering::Release);
    USER_QUEUE_IDLE.store(false, Ordering::Release);
    let mut registry = USER_CONTEXT_REGISTRY.lock();
    registry.ids = [first_id, second_id, third_id];
    registry.current = 0;
    registry.contexts = [
        Some(SavedUserContext {
            registers: SavedGpRegisters::default(),
            frame: first_frame,
        }),
        Some(SavedUserContext {
            registers: SavedGpRegisters::default(),
            frame: second_frame,
        }),
        Some(SavedUserContext {
            registers: SavedGpRegisters::default(),
            frame: third_frame,
        }),
    ];
    USER_PREEMPTION_READY.store(true, Ordering::Release);
}

pub fn user_preemption_switches() -> u64 {
    USER_PREEMPT_SWITCHES.load(Ordering::Acquire)
}

pub fn remove_user_context(id: u64) {
    let mut registry = USER_CONTEXT_REGISTRY.lock();
    for index in 0..USER_CONTEXT_CAPACITY {
        if registry.ids[index] == id {
            registry.ids[index] = 0;
            registry.contexts[index] = None;
        }
    }
}

pub fn select_user_context(id: u64) -> bool {
    let mut registry = USER_CONTEXT_REGISTRY.lock();
    for index in 0..USER_CONTEXT_CAPACITY {
        if registry.ids[index] == id && registry.contexts[index].is_some() {
            registry.current = index;
            return true;
        }
    }
    false
}

fn saved_user_context(id: u64) -> Option<SavedUserContext> {
    let registry = USER_CONTEXT_REGISTRY.lock();
    for index in 0..USER_CONTEXT_CAPACITY {
        if registry.ids[index] == id {
            return registry.contexts[index];
        }
    }
    None
}
