use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};

use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

use crate::{acpi::AcpiReport, apic, gdt, interrupts, memory::BootInfoFrameAllocator};

const MAX_CPUS: usize = 8;
const TRAMPOLINE_LIMIT: u64 = 0x10_0000;
const PML4_SLOT: usize = 16;
const STACK_SLOT: usize = 24;
const CPU_SLOT: usize = 32;
const ENTRY_SLOT: usize = 40;
const LONG_ENTRY_SLOT: usize = 48;
const ONLINE_SLOT: usize = 56;
const IDT_SLOT: usize = 64;
const STAGE_SLOT: usize = 72;
const PER_CPU_SLOT: usize = 80;
const LONG_MODE_OFFSET: u64 = 0x180;

const PER_CPU_OFFLINE: u8 = 0;
const PER_CPU_LONG_MODE: u8 = 1;
const PER_CPU_RUST_ENTRY: u8 = 2;
const PER_CPU_ONLINE: u8 = 3;

#[repr(C, align(64))]
pub struct PerCpuState {
    /// Pointer to this record, read through `gs:0` on its owning CPU.
    self_base: AtomicU64,
    apic_id: AtomicU64,
    bootstrap_stack_top: AtomicU64,
    stage: AtomicU8,
    gdt_tss_ready: AtomicBool,
    idt_ready: AtomicBool,
    online: AtomicBool,
    interrupt_nesting: AtomicU8,
    deferred_work: AtomicU64,
    current_process: AtomicU64,
}

impl PerCpuState {
    const fn new() -> Self {
        Self {
            self_base: AtomicU64::new(0),
            apic_id: AtomicU64::new(0),
            bootstrap_stack_top: AtomicU64::new(0),
            stage: AtomicU8::new(PER_CPU_OFFLINE),
            gdt_tss_ready: AtomicBool::new(false),
            idt_ready: AtomicBool::new(false),
            online: AtomicBool::new(false),
            interrupt_nesting: AtomicU8::new(0),
            deferred_work: AtomicU64::new(0),
            current_process: AtomicU64::new(0),
        }
    }

    fn reset(&self, self_base: u64, apic_id: u64, stack_top: u64) {
        self.self_base.store(self_base, Ordering::Release);
        self.apic_id.store(apic_id, Ordering::Release);
        self.bootstrap_stack_top.store(stack_top, Ordering::Release);
        self.stage.store(PER_CPU_LONG_MODE, Ordering::Release);
        self.gdt_tss_ready.store(false, Ordering::Release);
        self.idt_ready.store(false, Ordering::Release);
        self.online.store(false, Ordering::Release);
        self.interrupt_nesting.store(0, Ordering::Release);
        self.deferred_work.store(0, Ordering::Release);
        self.current_process.store(0, Ordering::Release);
    }

    fn enter_interrupt(&self) -> u8 {
        self.interrupt_nesting.fetch_add(1, Ordering::AcqRel) + 1
    }

    fn leave_interrupt(&self) -> u8 {
        self.interrupt_nesting.fetch_sub(1, Ordering::AcqRel) - 1
    }

    pub fn set_current_process(&self, id: u64) {
        self.current_process.store(id, Ordering::Release);
    }
}

/// Read-only view of one low-frame per-CPU record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerCpuSnapshot {
    pub apic_id: u64,
    pub bootstrap_stack_top: u64,
    pub stage: u8,
    pub gdt_tss_ready: bool,
    pub idt_ready: bool,
    pub online: bool,
    pub interrupt_nesting: u8,
    pub deferred_work: u64,
    pub current_process: u64,
}

/// SAFETY: `base` must be the permanent mapped virtual address of the frame
/// initialized by `initialize_per_cpu_frame`, and no caller may free or reuse it.
pub unsafe fn per_cpu_snapshot(base: u64, index: usize) -> Option<PerCpuSnapshot> {
    if index >= MAX_CPUS || base == 0 {
        return None;
    }
    let state = unsafe { &*((base as *const PerCpuState).add(index)) };
    Some(PerCpuSnapshot {
        apic_id: state.apic_id.load(Ordering::Acquire),
        bootstrap_stack_top: state.bootstrap_stack_top.load(Ordering::Acquire),
        stage: state.stage.load(Ordering::Acquire),
        gdt_tss_ready: state.gdt_tss_ready.load(Ordering::Acquire),
        idt_ready: state.idt_ready.load(Ordering::Acquire),
        online: state.online.load(Ordering::Acquire),
        interrupt_nesting: state.interrupt_nesting.load(Ordering::Acquire),
        deferred_work: state.deferred_work.load(Ordering::Acquire),
        current_process: state.current_process.load(Ordering::Acquire),
    })
}

/// SAFETY: `base` is a mapped virtual address for the permanent per-CPU frame.
unsafe fn per_cpu_state(base: u64, index: usize) -> &'static PerCpuState {
    unsafe { &*((base as *const PerCpuState).add(index)) }
}

/// SAFETY: each record is initialized before any AP can read it, and the frame
/// remains allocated for the lifetime of the experimental APs.
unsafe fn initialize_per_cpu_frame(base: u64) {
    let records = base as *mut PerCpuState;
    for index in 0..MAX_CPUS {
        unsafe { records.add(index).write(PerCpuState::new()) };
    }
}

/// Set `IA32_GS_BASE` to the owning low-memory per-CPU record.
fn install_gs_base(base: u64) -> bool {
    if base == 0 {
        return false;
    }
    let mut msr = x86_64::registers::model_specific::Msr::new(0xc000_0101);
    // SAFETY: `base` is an identity-mapped, aligned `PerCpuState` address
    // initialized by the BSP and kept allocated for this AP's lifetime.
    unsafe { msr.write(base) };
    true
}

/// Read the current CPU's record through the hardware GS base.
pub fn current_per_cpu() -> Option<&'static PerCpuState> {
    let msr = x86_64::registers::model_specific::Msr::new(0xc000_0101);
    // SAFETY: RDMSR of IA32_GS_BASE is a read-only CPU-local operation; zero
    // denotes the BSP or an uninitialized CPU and is handled without memory IO.
    let base = unsafe { msr.read() };
    if base == 0 {
        None
    } else {
        Some(unsafe { &*(base as *const PerCpuState) })
    }
}

pub fn running_on_ap() -> bool {
    current_per_cpu()
        .map(|state| state.apic_id.load(Ordering::Acquire) != 0)
        .unwrap_or(false)
}

pub fn defer_work(units: u64) -> bool {
    let Some(state) = current_per_cpu() else {
        return false;
    };
    state.deferred_work.fetch_add(units, Ordering::AcqRel);
    true
}

pub fn interrupt_guard() -> InterruptNestingGuard {
    InterruptNestingGuard {
        active: current_per_cpu().map(|state| {
            state.enter_interrupt();
        }),
    }
}

pub struct InterruptNestingGuard {
    active: Option<()>,
}

impl Drop for InterruptNestingGuard {
    fn drop(&mut self) {
        if self.active.is_some() {
            if let Some(state) = current_per_cpu() {
                state.leave_interrupt();
            }
        }
    }
}

fn count_per_cpu_online(base: u64, limit: usize) -> u32 {
    if base == 0 {
        return 0;
    }
    (0..limit.min(MAX_CPUS))
        .filter(|&index| unsafe { per_cpu_state(base, index).online.load(Ordering::Acquire) })
        .count() as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmpReport {
    pub discovered: u32,
    pub attempted: u32,
    pub online: u32,
    pub trampoline_installed: bool,
    pub last_stage: u8,
    pub kernel_pages_mapped: u64,
    pub kernel_coverage_complete: bool,
    pub kernel_coverage_error: u8,
    pub per_cpu_ready: u32,
}

pub fn unavailable_report(report: AcpiReport) -> SmpReport {
    SmpReport {
        discovered: report.cpus,
        attempted: 0,
        online: 1,
        trampoline_installed: false,
        last_stage: 0,
        kernel_pages_mapped: 0,
        kernel_coverage_complete: false,
        kernel_coverage_error: 0,
        per_cpu_ready: 1,
    }
}

pub fn ap_entry_runtime_address() -> u64 {
    ap_entry as *const () as usize as u64
}

pub fn bring_up(
    report: AcpiReport,
    physical_memory_offset: VirtAddr,
    mapper: &mut OffsetPageTable<'static>,
    allocator: &mut BootInfoFrameAllocator,
    kernel_addr: u64,
    kernel_len: u64,
    kernel_image_offset: u64,
) -> SmpReport {
    let discovered = report.cpus;
    if !apic::enabled() || discovered <= 1 {
        return SmpReport {
            discovered,
            attempted: 0,
            online: 1,
            trampoline_installed: false,
            last_stage: 0,
            kernel_pages_mapped: 0,
            kernel_coverage_complete: false,
            kernel_coverage_error: 0,
            per_cpu_ready: 0,
        };
    }

    let Some(trampoline_frame) = allocator.allocate_frame_below(TRAMPOLINE_LIMIT) else {
        return SmpReport {
            discovered,
            attempted: 0,
            online: 1,
            trampoline_installed: false,
            last_stage: 0,
            kernel_pages_mapped: 0,
            kernel_coverage_complete: false,
            kernel_coverage_error: 0,
            per_cpu_ready: 0,
        };
    };
    let trampoline_physical = trampoline_frame.start_address().as_u64();
    let trampoline_page = Page::<Size4KiB>::containing_address(VirtAddr::new(trampoline_physical));
    // SAFETY: the reserved low-memory frame is not mapped elsewhere by this
    // kernel path, and the mapper is the active root used by the BSP/APs.
    let mapped = unsafe {
        mapper.map_to(
            trampoline_page,
            trampoline_frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            allocator,
        )
    };
    if mapped.is_err() {
        return SmpReport {
            discovered,
            attempted: 0,
            online: 1,
            trampoline_installed: false,
            last_stage: 0,
            kernel_pages_mapped: 0,
            kernel_coverage_complete: false,
            kernel_coverage_error: 0,
            per_cpu_ready: 0,
        };
    }
    // Use the tail of the already identity-mapped trampoline page for the
    // first handshake. It is guaranteed to be visible to the AP before any
    // additional page-table coverage is established.
    let online_physical = trampoline_physical + 0x3f0;
    let online_virtual = physical_memory_offset
        .as_u64()
        .saturating_add(online_physical);
    unsafe { core::ptr::write_bytes(online_virtual as *mut u8, 0, 16) };
    if !install_trampoline(trampoline_physical, physical_memory_offset) {
        return SmpReport {
            discovered,
            attempted: 0,
            online: 1,
            trampoline_installed: false,
            last_stage: 0,
            kernel_pages_mapped: 0,
            kernel_coverage_complete: false,
            kernel_coverage_error: 0,
            per_cpu_ready: 0,
        };
    }

    let (root, _) = Cr3::read();
    let (kernel_pages_mapped, kernel_coverage_complete, kernel_coverage_error) =
        ensure_kernel_coverage(
            mapper,
            allocator,
            kernel_addr,
            kernel_len,
            kernel_image_offset,
        );
    let idt_physical = {
        #[cfg(feature = "smp-experimental")]
        {
            let Some(idt_frame) = allocator.allocate_frame_below(TRAMPOLINE_LIMIT) else {
                return SmpReport {
                    discovered,
                    attempted: 0,
                    online: 1,
                    trampoline_installed: true,
                    last_stage: 0,
                    kernel_pages_mapped,
                    kernel_coverage_complete,
                    kernel_coverage_error,
                    per_cpu_ready: 0,
                };
            };
            let physical = idt_frame.start_address().as_u64();
            let idt_page = Page::<Size4KiB>::containing_address(VirtAddr::new(physical));
            let idt_mapping = unsafe {
                mapper.map_to(
                    idt_page,
                    idt_frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    allocator,
                )
            };
            if idt_mapping.is_err() {
                return SmpReport {
                    discovered,
                    attempted: 0,
                    online: 1,
                    trampoline_installed: true,
                    last_stage: 0,
                    kernel_pages_mapped,
                    kernel_coverage_complete,
                    kernel_coverage_error,
                    per_cpu_ready: 0,
                };
            }
            // SAFETY: this permanent frame is owned by the active root and is
            // never freed while an AP can receive interrupts.
            if !unsafe { interrupts::build_idt_at(VirtAddr::new(physical)) } {
                return SmpReport {
                    discovered,
                    attempted: 0,
                    online: 1,
                    trampoline_installed: true,
                    last_stage: 0,
                    kernel_pages_mapped,
                    kernel_coverage_complete,
                    kernel_coverage_error,
                    per_cpu_ready: 0,
                };
            }
            physical
        }
        #[cfg(not(feature = "smp-experimental"))]
        {
            0
        }
    };
    let per_cpu_physical = {
        #[cfg(feature = "smp-experimental")]
        {
            let Some(frame) = allocator.allocate_frame_below(TRAMPOLINE_LIMIT) else {
                return SmpReport {
                    discovered,
                    attempted: 0,
                    online: 1,
                    trampoline_installed: true,
                    last_stage: 0,
                    kernel_pages_mapped,
                    kernel_coverage_complete,
                    kernel_coverage_error,
                    per_cpu_ready: 0,
                };
            };
            let physical = frame.start_address().as_u64();
            let page = Page::<Size4KiB>::containing_address(VirtAddr::new(physical));
            let mapping = unsafe {
                mapper.map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    allocator,
                )
            };
            if mapping.is_err() {
                return SmpReport {
                    discovered,
                    attempted: 0,
                    online: 1,
                    trampoline_installed: true,
                    last_stage: 0,
                    kernel_pages_mapped,
                    kernel_coverage_complete,
                    kernel_coverage_error,
                    per_cpu_ready: 0,
                };
            }
            // SAFETY: the newly allocated frame is mapped through the BSP's
            // physical-memory offset and remains allocated for all APs.
            unsafe { initialize_per_cpu_frame(physical_memory_offset.as_u64() + physical) };
            physical
        }
        #[cfg(not(feature = "smp-experimental"))]
        {
            0
        }
    };
    let per_cpu_virtual = physical_memory_offset.as_u64() + per_cpu_physical;
    if per_cpu_physical != 0 {
        // The BSP owns record 0 as well. Its GS base uses the mapped virtual
        // alias, while APs use the identity-mapped physical alias of their
        // records; both aliases refer to the same permanent frame.
        let bsp_stack = gdt::kernel_stack_top().as_u64();
        // SAFETY: the frame was initialized above and is permanently mapped.
        unsafe {
            let bsp = per_cpu_state(per_cpu_virtual, 0);
            bsp.reset(per_cpu_virtual, 0, bsp_stack);
            bsp.gdt_tss_ready.store(true, Ordering::Release);
            bsp.idt_ready.store(true, Ordering::Release);
            bsp.stage.store(PER_CPU_ONLINE, Ordering::Release);
            bsp.online.store(true, Ordering::Release);
        }
        if !install_gs_base(per_cpu_virtual) {
            return SmpReport {
                discovered,
                attempted: 0,
                online: 1,
                trampoline_installed: true,
                last_stage: 0,
                kernel_pages_mapped,
                kernel_coverage_complete,
                kernel_coverage_error,
                per_cpu_ready: 0,
            };
        }
    }
    let mut attempted = 0;
    let mut online = 1;
    let mut last_stage = 0;
    let vector = (trampoline_physical >> 12) as u8;
    for index in 0..(report.cpus as usize).min(MAX_CPUS) {
        let local_id = report.cpu_ids[index];
        if local_id == 0 {
            continue;
        }
        attempted += 1;
        let Some(stack_frame) = allocator.allocate_frame_below(TRAMPOLINE_LIMIT) else {
            continue;
        };
        let stack_page = Page::<Size4KiB>::containing_address(VirtAddr::new(
            stack_frame.start_address().as_u64(),
        ));
        let stack_mapping = unsafe {
            mapper.map_to(
                stack_page,
                stack_frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                allocator,
            )
        };
        if stack_mapping.is_err() {
            continue;
        }
        let stack_top = VirtAddr::new(stack_frame.start_address().as_u64() + 4096 - 16);
        if per_cpu_physical != 0 {
            // SAFETY: this is the mapped BSP alias of the permanent per-CPU frame.
            unsafe {
                let record_base = per_cpu_physical
                    .checked_add((index * core::mem::size_of::<PerCpuState>()) as u64)
                    .expect("per-CPU record address overflow");
                per_cpu_state(per_cpu_virtual, index).reset(
                    record_base,
                    local_id as u64,
                    stack_top.as_u64(),
                )
            };
        }
        if !patch_trampoline(
            trampoline_physical,
            physical_memory_offset,
            root.start_address().as_u64(),
            stack_top,
            local_id as u64,
            ap_entry as *const () as usize as u64,
            trampoline_physical + LONG_MODE_OFFSET,
            online_physical,
            idt_physical,
            online_physical + 1,
            per_cpu_physical,
        ) {
            continue;
        }
        if !apic::start_ap(local_id, vector) {
            last_stage = 0xfe;
            continue;
        }
        let (ap_online, stage) = wait_online(physical_memory_offset, online_physical, index);
        last_stage = stage;
        if ap_online {
            online += 1;
        }
    }

    SmpReport {
        discovered,
        attempted,
        online,
        trampoline_installed: true,
        last_stage,
        kernel_pages_mapped,
        kernel_coverage_complete,
        kernel_coverage_error,
        per_cpu_ready: count_per_cpu_online(per_cpu_virtual, report.cpus as usize),
    }
}

fn ensure_kernel_coverage(
    mapper: &mut OffsetPageTable<'static>,
    allocator: &mut BootInfoFrameAllocator,
    kernel_addr: u64,
    kernel_len: u64,
    kernel_image_offset: u64,
) -> (u64, bool, u8) {
    let pages = (kernel_len.saturating_add(4095)) / 4096;
    let mut mapped = 0;
    for page_index in 0..pages {
        let link = page_index.saturating_mul(4096);
        let virtual_address = match kernel_image_offset.checked_add(link) {
            Some(value) => VirtAddr::new(value),
            None => return (mapped, false, 0),
        };
        if mapper.translate_addr(virtual_address).is_some() {
            continue;
        }
        let physical = match kernel_addr.checked_add(link) {
            Some(value) => PhysAddr::new(value),
            None => return (mapped, false, 0),
        };
        let page = Page::<Size4KiB>::containing_address(virtual_address);
        let frame = PhysFrame::<Size4KiB>::containing_address(physical);
        let result = unsafe {
            mapper.map_to(
                page,
                frame,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                allocator,
            )
        };
        if let Err(error) = result {
            let code = match error {
                x86_64::structures::paging::mapper::MapToError::FrameAllocationFailed => 1,
                x86_64::structures::paging::mapper::MapToError::ParentEntryHugePage => 2,
                x86_64::structures::paging::mapper::MapToError::PageAlreadyMapped(_) => 3,
            };
            return (mapped, false, code);
        }
        mapped += 1;
    }
    (mapped, true, 0)
}

fn install_trampoline(physical: u64, offset: VirtAddr) -> bool {
    let bytes = include_bytes!(env!("AP_TRAMPOLINE_BIN"));
    if bytes.len() > 4096 || physical >= TRAMPOLINE_LIMIT {
        return false;
    }
    let destination = match offset.as_u64().checked_add(physical) {
        Some(address) => address as *mut u8,
        None => return false,
    };
    unsafe {
        core::ptr::write_bytes(destination, 0, 4096);
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len());
    }
    true
}

fn patch_trampoline(
    physical: u64,
    offset: VirtAddr,
    root: u64,
    stack: VirtAddr,
    cpu_id: u64,
    entry: u64,
    long_entry: u64,
    online: u64,
    idt: u64,
    stage: u64,
    per_cpu: u64,
) -> bool {
    let base = match offset.as_u64().checked_add(physical) {
        Some(address) => address,
        None => return false,
    };
    unsafe {
        let slots = base as *mut u64;
        slots.add(PML4_SLOT / 8).write_volatile(root);
        slots.add(STACK_SLOT / 8).write_volatile(stack.as_u64());
        slots.add(CPU_SLOT / 8).write_volatile(cpu_id);
        slots.add(ENTRY_SLOT / 8).write_volatile(entry);
        slots.add(LONG_ENTRY_SLOT / 8).write_volatile(long_entry);
        slots.add(ONLINE_SLOT / 8).write_volatile(online);
        slots.add(IDT_SLOT / 8).write_volatile(idt);
        slots.add(STAGE_SLOT / 8).write_volatile(stage);
        slots.add(PER_CPU_SLOT / 8).write_volatile(per_cpu);
    }
    true
}

fn wait_online(offset: VirtAddr, online_physical: u64, _index: usize) -> (bool, u8) {
    let online_offset = offset.as_u64().saturating_add(online_physical);
    let stage_physical = online_physical.saturating_add(1);
    let stage_offset = offset.as_u64().saturating_add(stage_physical);
    let mut last = 0;
    for _ in 0..500_000_000 {
        let stage = unsafe { core::ptr::read_volatile(stage_offset as *const u8) };
        let identity_stage = unsafe { core::ptr::read_volatile(stage_physical as *const u8) };
        last = stage.max(identity_stage);
        if stage == 3 || identity_stage == 3 {
            return (true, 3);
        }
        let online = unsafe { core::ptr::read_volatile(online_offset as *const u8) };
        let identity_online = unsafe { core::ptr::read_volatile(online_physical as *const u8) };
        if online == 3 || identity_online == 3 {
            return (true, 3);
        }
        core::hint::spin_loop();
    }
    (false, last)
}

#[unsafe(no_mangle)]
extern "C" fn ap_entry(
    cpu_id: u64,
    stack_top: u64,
    idt_addr: u64,
    stage_addr: u64,
    per_cpu_base: u64,
) -> ! {
    let index = cpu_id as usize;
    if index < MAX_CPUS && per_cpu_base != 0 {
        // The stage byte is a low identity-mapped diagnostic contract shared
        // with the trampoline; it avoids hiding AP progress in high-half state.
        unsafe { core::ptr::write_volatile(stage_addr as *mut u8, 2) };
        // SAFETY: bring_up initialized this permanent low frame before SIPI.
        let state = unsafe { per_cpu_state(per_cpu_base, index) };
        let state_base = per_cpu_base
            .checked_add((index * core::mem::size_of::<PerCpuState>()) as u64)
            .unwrap_or(0);
        if !install_gs_base(state_base) {
            loop {
                unsafe { asm!("cli; hlt", options(nomem, nostack, preserves_flags)) };
            }
        }
        state.stage.store(PER_CPU_RUST_ENTRY, Ordering::Release);
        let mut descriptors = gdt::ApDescriptorState::new(VirtAddr::new(stack_top));
        if !descriptors.load() {
            loop {
                unsafe { asm!("cli; hlt", options(nomem, nostack, preserves_flags)) };
            }
        }
        state.gdt_tss_ready.store(true, Ordering::Release);
        // SAFETY: bring_up allocated, initialized, and identity-mapped this
        // permanent frame; it is not freed while the AP is running.
        if !unsafe { interrupts::init_ap_at(VirtAddr::new(idt_addr)) } {
            loop {
                unsafe { asm!("cli; hlt", options(nomem, nostack, preserves_flags)) };
            }
        }
        state.idt_ready.store(true, Ordering::Release);
        unsafe { core::ptr::write_volatile(stage_addr as *mut u8, 3) };
        state.stage.store(PER_CPU_ONLINE, Ordering::Release);
        state.online.store(true, Ordering::Release);
    }
    unsafe {
        asm!("sti", options(nomem, nostack, preserves_flags));
    }
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[allow(dead_code)]
fn _phys(physical: u64) -> PhysAddr {
    PhysAddr::new(physical)
}
