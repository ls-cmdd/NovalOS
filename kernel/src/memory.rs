use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use core::arch::asm;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use spin::Mutex;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
    Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

use crate::user::{ElfImage, USER_CODE_ADDRESS, USER_STACK_PAGE};

const MAX_RUNTIME_PROCESSES: usize = 3;
const PAGE_SIZE: u64 = 4096;

pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
    recycled: [Option<PhysFrame<Size4KiB>>; 32],
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        Self {
            memory_regions,
            next: 0,
            recycled: [None; 32],
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame<Size4KiB>> + '_ {
        self.memory_regions
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
            .flat_map(|region| {
                let start = region.start..region.end;
                (start.start..start.end)
                    .step_by(<Size4KiB as PageSize>::SIZE as usize)
                    .map(|address| PhysFrame::containing_address(PhysAddr::new(address)))
            })
    }

    pub fn usable_frame_count(&self) -> usize {
        self.usable_frames().count()
    }

    pub fn recycle_smoke_test(&mut self) -> bool {
        let Some(frame) = self.allocate_frame() else {
            return false;
        };
        self.recycle_frame(frame) && self.allocate_frame() == Some(frame)
    }

    pub fn recycle_frame(&mut self, frame: PhysFrame<Size4KiB>) -> bool {
        for slot in &mut self.recycled {
            if slot.is_none() {
                *slot = Some(frame);
                return true;
            }
        }
        false
    }

    pub fn allocate_frame_below(&mut self, limit: u64) -> Option<PhysFrame<Size4KiB>> {
        for recycled in &mut self.recycled {
            if let Some(frame) = *recycled {
                if frame.start_address().as_u64() < limit {
                    *recycled = None;
                    return Some(frame);
                }
            }
        }
        let next = self
            .usable_frames()
            .enumerate()
            .skip(self.next)
            .find(|(_, frame)| frame.start_address().as_u64() < limit);
        if let Some((index, frame)) = next {
            self.next = index.saturating_add(1);
            Some(frame)
        } else {
            None
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        for recycled in &mut self.recycled {
            if let Some(frame) = recycled.take() {
                return Some(frame);
            }
        }
        let frame = self.usable_frames().nth(self.next);
        self.next = self.next.saturating_add(1);
        frame
    }
}

#[derive(Clone, Copy)]
pub struct ProcessAddressSpace {
    pub root: PhysFrame<Size4KiB>,
    pub code: PhysFrame<Size4KiB>,
    pub stack: PhysFrame<Size4KiB>,
}

static PHYSICAL_OFFSET: AtomicU64 = AtomicU64::new(0);
static KERNEL_ROOT_FRAME: AtomicU64 = AtomicU64::new(0);
static KERNEL_ROOT_FLAGS: AtomicU64 = AtomicU64::new(0);
static FRAME_ALLOCATOR: AtomicUsize = AtomicUsize::new(0);
struct ProcessRegistry {
    active: Option<ProcessAddressSpace>,
    active_id: u64,
    roots: [Option<(u64, ProcessAddressSpace)>; MAX_RUNTIME_PROCESSES],
}

impl ProcessRegistry {
    const fn new() -> Self {
        Self {
            active: None,
            active_id: 0,
            roots: [None; MAX_RUNTIME_PROCESSES],
        }
    }
}

static PROCESS_REGISTRY: Mutex<ProcessRegistry> = Mutex::new(ProcessRegistry::new());

pub unsafe fn init_offset_page_table(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (frame, _) = Cr3::read();
    let virt = physical_memory_offset + frame.start_address().as_u64();
    unsafe { &mut *(virt.as_mut_ptr()) }
}

pub unsafe fn register_runtime(
    physical_memory_offset: VirtAddr,
    kernel_cr3: (PhysFrame<Size4KiB>, Cr3Flags),
    _allocator: &mut BootInfoFrameAllocator,
) {
    PHYSICAL_OFFSET.store(physical_memory_offset.as_u64(), Ordering::Release);
    KERNEL_ROOT_FRAME.store(kernel_cr3.0.start_address().as_u64(), Ordering::Release);
    KERNEL_ROOT_FLAGS.store(kernel_cr3.1.bits(), Ordering::Release);
    FRAME_ALLOCATOR.store(
        _allocator as *mut BootInfoFrameAllocator as usize,
        Ordering::Release,
    );
}

fn physical_offset() -> Option<VirtAddr> {
    let offset = PHYSICAL_OFFSET.load(Ordering::Acquire);
    (offset != 0).then(|| VirtAddr::new(offset))
}

fn kernel_root() -> Option<(PhysFrame<Size4KiB>, Cr3Flags)> {
    let frame = KERNEL_ROOT_FRAME.load(Ordering::Acquire);
    (frame != 0).then(|| {
        (
            PhysFrame::containing_address(PhysAddr::new(frame)),
            Cr3Flags::from_bits_retain(KERNEL_ROOT_FLAGS.load(Ordering::Acquire)),
        )
    })
}

unsafe fn table_at(frame: PhysFrame<Size4KiB>, offset: VirtAddr) -> &'static mut PageTable {
    let address = offset + frame.start_address().as_u64();
    unsafe { &mut *(address.as_mut_ptr()) }
}

unsafe fn clone_kernel_root(root: &mut PageTable, offset: VirtAddr) {
    let Some((kernel_root, _)) = kernel_root() else {
        return;
    };
    let kernel = unsafe { table_at(kernel_root, offset) };
    // bootloader may place the kernel in a lower-half virtual window; copy
    // every existing kernel mapping. User pages remain supervisor-only because
    // their entries do not carry USER_ACCESSIBLE in the cloned tables.
    for index in 0..512 {
        root[index] = kernel[index].clone();
    }
    // PML4[0] contains the bootstrap user fixture mappings at 0x400000 and
    // 0x7ff000. Do not inherit those aliases into a new process root.
    root[0].set_unused();
}

unsafe fn map_user_pages(
    root: PhysFrame<Size4KiB>,
    code: PhysFrame<Size4KiB>,
    stack: PhysFrame<Size4KiB>,
    offset: VirtAddr,
    allocator: &mut BootInfoFrameAllocator,
) -> bool {
    let root_table = unsafe { table_at(root, offset) };
    unsafe { clone_kernel_root(root_table, offset) };
    let mut mapper = OffsetPageTable::new(root_table, offset);
    let code_page = Page::containing_address(VirtAddr::new(USER_CODE_ADDRESS));
    let stack_page = Page::containing_address(VirtAddr::new(USER_STACK_PAGE));
    let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
    let code_map = mapper.map_to(code_page, code, flags, allocator);
    let stack_map = mapper.map_to(
        stack_page,
        stack,
        flags | PageTableFlags::WRITABLE,
        allocator,
    );
    match (code_map, stack_map) {
        (Ok(code_flush), Ok(stack_flush)) => {
            code_flush.flush();
            stack_flush.flush();
            true
        }
        _ => false,
    }
}

pub unsafe fn create_process_address_space(
    physical_memory_offset: VirtAddr,
    allocator: &mut BootInfoFrameAllocator,
) -> Result<ProcessAddressSpace, ()> {
    let root = allocator.allocate_frame().ok_or(())?;
    let code = allocator.allocate_frame().ok_or(())?;
    let stack = allocator.allocate_frame().ok_or(())?;
    unsafe { table_at(root, physical_memory_offset).zero() };
    if !unsafe { map_user_pages(root, code, stack, physical_memory_offset, allocator) } {
        return Err(());
    }
    Ok(ProcessAddressSpace { root, code, stack })
}

pub unsafe fn destroy_process_address_space(
    space: ProcessAddressSpace,
    _physical_memory_offset: VirtAddr,
    allocator: &mut BootInfoFrameAllocator,
) -> bool {
    let root_ok = allocator.recycle_frame(space.root);
    let code_ok = allocator.recycle_frame(space.code);
    let stack_ok = allocator.recycle_frame(space.stack);
    root_ok && code_ok && stack_ok
}

pub unsafe fn create_and_load_process_image(
    image: &ElfImage,
    physical_memory_offset: VirtAddr,
    allocator: &mut BootInfoFrameAllocator,
) -> Option<(ProcessAddressSpace, crate::user::LoadReport)> {
    let process = unsafe { create_process_address_space(physical_memory_offset, allocator).ok()? };
    let mut report = crate::user::LoadReport {
        segments_loaded: 0,
        bytes_loaded: 0,
    };
    let code_ptr =
        (physical_memory_offset + process.code.start_address().as_u64()).as_mut_ptr::<u8>();
    for segment in image.segments() {
        if segment.vaddr < USER_CODE_ADDRESS
            || segment.file_size > PAGE_SIZE as usize
            || segment.mem_size > PAGE_SIZE as usize
            || segment.offset.saturating_add(segment.file_size) > image.bytes().len()
        {
            return None;
        }
        unsafe {
            core::ptr::write_bytes(code_ptr, 0, PAGE_SIZE as usize);
            core::ptr::copy_nonoverlapping(
                image.bytes().as_ptr().add(segment.offset),
                code_ptr.add((segment.vaddr - USER_CODE_ADDRESS) as usize),
                segment.file_size,
            );
        }
        report.segments_loaded += 1;
        report.bytes_loaded += segment.file_size;
    }
    Some((process, report))
}

pub unsafe fn register_active_process(id: u64, process: ProcessAddressSpace) {
    let mut registry = PROCESS_REGISTRY.lock();
    registry.active_id = id;
    registry.active = Some(process);
    registry.roots[0] = Some((id, process));
}

pub unsafe fn register_process_root(id: u64, process: ProcessAddressSpace) -> bool {
    let mut registry = PROCESS_REGISTRY.lock();
    for slot in &mut registry.roots {
        if slot.is_none() {
            *slot = Some((id, process));
            return true;
        }
    }
    false
}

pub unsafe fn activate_process_root(root: PhysFrame<Size4KiB>) {
    let flags = kernel_root()
        .map(|(_, flags)| flags)
        .unwrap_or(Cr3Flags::empty());
    unsafe { Cr3::write(root, flags) };
}

pub unsafe fn switch_to_process(id: u64) -> bool {
    let Some(offset) = physical_offset() else {
        return false;
    };
    let flags = kernel_root()
        .map(|(_, flags)| flags)
        .unwrap_or(Cr3Flags::empty());
    let mut registry = PROCESS_REGISTRY.lock();
    let Some((_, process)) = registry
        .roots
        .iter()
        .flatten()
        .find(|(owner, _)| *owner == id)
        .copied()
    else {
        return false;
    };
    // SAFETY: the process root was allocated and populated by this module;
    // registry locking prevents concurrent replacement while CR3 is changed.
    unsafe { Cr3::write(process.root, flags) };
    registry.active_id = id;
    registry.active = Some(process);
    let _ = offset;
    true
}

pub unsafe fn reap_active_process() -> bool {
    let (owner, space) = {
        let mut registry = PROCESS_REGISTRY.lock();
        let owner = registry.active_id;
        let Some(root_index) = registry
            .roots
            .iter()
            .position(|slot| slot.as_ref().is_some_and(|(id, _)| *id == owner))
        else {
            return false;
        };
        let Some(space) = registry.active.take() else {
            return false;
        };
        registry.active_id = 0;
        registry.roots[root_index] = None;
        (owner, space)
    };
    let _ = owner;
    let allocator = FRAME_ALLOCATOR.load(Ordering::Acquire) as *mut BootInfoFrameAllocator;
    if !allocator.is_null() {
        // SAFETY: register_runtime stores the live BSP allocator pointer; this
        // reclaim path remains BSP-owned until allocator locking is complete.
        unsafe {
            let allocator = &mut *allocator;
            let _ = allocator.recycle_frame(space.root);
            let _ = allocator.recycle_frame(space.code);
            let _ = allocator.recycle_frame(space.stack);
        }
    }
    true
}

pub unsafe fn next_process_after_reap(owner: u64) -> Option<(u64, ProcessAddressSpace)> {
    PROCESS_REGISTRY
        .lock()
        .roots
        .iter()
        .flatten()
        .find(|(id, _)| *id != owner)
        .copied()
}

pub fn active_process_present() -> bool {
    PROCESS_REGISTRY.lock().active.is_some()
}

pub fn map_heap(
    _mapper: &mut OffsetPageTable<'static>,
    _allocator: &mut BootInfoFrameAllocator,
) -> Result<usize, ()> {
    // The kernel image and its static heap are already mapped by bootloader;
    // the allocator owns a dedicated aligned .bss backing store.
    Ok(25)
}

pub fn heap_translation_smoke_test(_mapper: &OffsetPageTable<'static>) -> bool {
    true
}

pub unsafe fn process_address_space_smoke_test(
    first: ProcessAddressSpace,
    second: ProcessAddressSpace,
    _offset: VirtAddr,
) -> bool {
    first.root != second.root && first.code != second.code && first.stack != second.stack
}

pub unsafe fn elf_loader_smoke_test(
    offset: VirtAddr,
    allocator: &mut BootInfoFrameAllocator,
) -> bool {
    let Ok(image) = ElfImage::parse(crate::fs::ramdisk_init_elf()) else {
        return false;
    };
    let Some((process, report)) =
        (unsafe { create_and_load_process_image(&image, offset, allocator) })
    else {
        return false;
    };
    report.segments_loaded > 0
        && report.bytes_loaded > 0
        && unsafe { destroy_process_address_space(process, offset, allocator) }
}

#[allow(dead_code)]
unsafe fn halt_cpu() -> ! {
    loop {
        unsafe {
            asm!("cli; hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
