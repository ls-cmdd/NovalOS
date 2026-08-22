use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::{Mutex, Once};
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::Segment;
use x86_64::structures::{
    gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
    tss::TaskStateSegment,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const PROCESS_STACK_SIZE: usize = 16 * 1024;
const PROCESS_STACKS: usize = 3;

#[repr(align(16))]
struct ProcessKernelStack([u8; PROCESS_STACK_SIZE]);

static TSS: Once<TaskStateSegment> = Once::new();
static KERNEL_STACK_TOP: AtomicU64 = AtomicU64::new(0);
static mut USER_KERNEL_STACKS: [ProcessKernelStack; PROCESS_STACKS] =
    [const { ProcessKernelStack([0; PROCESS_STACK_SIZE]) }; PROCESS_STACKS];
static GDT: Once<(GlobalDescriptorTable, Selectors)> = Once::new();
static PROCESS_STACK_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy)]
pub struct Selectors {
    pub code: SegmentSelector,
    pub data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    tss: SegmentSelector,
}

pub fn init() {
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        let stack_top = bootstrap_stack_top();
        KERNEL_STACK_TOP.store(stack_top.as_u64(), Ordering::Release);
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_top;
        tss.privilege_stack_table[0] = stack_top;
        tss
    });
    let (gdt, selectors) = GDT.call_once(|| {
        let mut gdt = GlobalDescriptorTable::new();
        let code = gdt.append(Descriptor::kernel_code_segment());
        let data = gdt.append(Descriptor::kernel_data_segment());
        let user_data = gdt.append(Descriptor::user_data_segment());
        let user_code = gdt.append(Descriptor::user_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(tss));
        (
            gdt,
            Selectors {
                code,
                data,
                user_code,
                user_data,
                tss: tss_selector,
            },
        )
    });
    gdt.load();
    unsafe {
        x86_64::instructions::segmentation::CS::set_reg(selectors.code);
        x86_64::instructions::tables::load_tss(selectors.tss);
    }
}

/// AP bootstrap descriptor state owned by the AP entry stack frame.
///
/// The BSP's `Once`-backed GDT/TSS objects live in the high-half kernel image.
/// An AP must not dereference those objects until its page-table coverage and
/// per-CPU mappings are proven. This state therefore lives on the AP's
/// low-memory bootstrap stack and is kept alive while the AP is halted.
pub struct ApDescriptorState {
    tss: TaskStateSegment,
    gdt: GlobalDescriptorTable,
    code: SegmentSelector,
    tss_selector: SegmentSelector,
    loaded: bool,
}

impl ApDescriptorState {
    pub fn new(stack_top: VirtAddr) -> Self {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_top;
        tss.privilege_stack_table[0] = stack_top;
        Self {
            tss,
            gdt: GlobalDescriptorTable::new(),
            code: SegmentSelector::new(0, x86_64::PrivilegeLevel::Ring0),
            tss_selector: SegmentSelector::new(0, x86_64::PrivilegeLevel::Ring0),
            loaded: false,
        }
    }

    pub fn load(&mut self) -> bool {
        if self.loaded {
            return true;
        }
        let code = self.gdt.append(Descriptor::kernel_code_segment());
        let _data = self.gdt.append(Descriptor::kernel_data_segment());
        let _user_data = self.gdt.append(Descriptor::user_data_segment());
        let _user_code = self.gdt.append(Descriptor::user_code_segment());
        // SAFETY: the AP remains in this function's terminal loop after the
        // descriptor is loaded, so the stack-owned TSS address stays valid.
        let tss: &'static TaskStateSegment = unsafe { &*(&self.tss as *const _) };
        let tss_selector = self.gdt.append(Descriptor::tss_segment(tss));
        // SAFETY: `self` is allocated in the AP entry stack frame, never moved
        // after this call, and the AP enters a terminal halt loop while this
        // descriptor table remains live.
        let gdt: &'static GlobalDescriptorTable = unsafe { &*(&self.gdt as *const _) };
        gdt.load();
        unsafe {
            x86_64::instructions::segmentation::CS::set_reg(code);
            x86_64::instructions::tables::load_tss(tss_selector);
        }
        self.code = code;
        self.tss_selector = tss_selector;
        self.loaded = true;
        true
    }
}

pub fn user_selectors() -> Selectors {
    GDT.get()
        .map(|(_, selectors)| *selectors)
        .unwrap_or_else(|| Selectors {
            code: SegmentSelector::new(0, x86_64::PrivilegeLevel::Ring0),
            data: SegmentSelector::new(0, x86_64::PrivilegeLevel::Ring0),
            user_code: SegmentSelector::new(0, x86_64::PrivilegeLevel::Ring3),
            user_data: SegmentSelector::new(0, x86_64::PrivilegeLevel::Ring3),
            tss: SegmentSelector::new(0, x86_64::PrivilegeLevel::Ring0),
        })
}

fn bootstrap_stack_top() -> VirtAddr {
    let stack: u64;
    unsafe { asm!("mov {}, rsp", out(reg) stack, options(nomem, nostack, preserves_flags)) };
    VirtAddr::new(stack & !0xF)
}

pub fn kernel_stack_top() -> VirtAddr {
    VirtAddr::new(KERNEL_STACK_TOP.load(Ordering::Acquire))
}

pub fn set_process_kernel_stack(process_id: u64) -> bool {
    let _stack_lock = PROCESS_STACK_LOCK.lock();
    if !(1..=(PROCESS_STACKS as u64)).contains(&process_id) {
        return false;
    }
    let index = (process_id - 1) as usize;
    let bottom = unsafe { core::ptr::addr_of_mut!(USER_KERNEL_STACKS[index].0) as *mut u8 };
    let top = unsafe { bottom.add(PROCESS_STACK_SIZE) } as u64 & !0xF;
    KERNEL_STACK_TOP.store(top, Ordering::Release);
    let Some(tss) = TSS.get() else {
        return false;
    };
    let tss_ptr = tss as *const TaskStateSegment as *mut TaskStateSegment;
    unsafe {
        (*tss_ptr).privilege_stack_table[0] = VirtAddr::new(top);
        (*tss_ptr).interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = VirtAddr::new(top);
    }
    true
}
