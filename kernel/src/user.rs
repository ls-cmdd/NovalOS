use alloc::vec::Vec;
use core::arch::asm;
use x86_64::VirtAddr;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB, Translate,
};

pub const USER_CODE_ADDRESS: u64 = 0x400000;
pub const USER_STACK_PAGE: u64 = 0x7FF000;
pub const USER_STACK_TOP: u64 = USER_STACK_PAGE + 0x1000;
pub const USER_GUARD_PAGE: u64 = 0x7FE000;

#[derive(Clone, Copy)]
pub struct UserMapping {
    pub code: PhysFrame<Size4KiB>,
    pub stack: PhysFrame<Size4KiB>,
}

pub fn map_user_code_and_stack(
    mapper: &mut OffsetPageTable<'static>,
    allocator: &mut impl FrameAllocator<Size4KiB>,
    _physical_memory_offset: VirtAddr,
) -> Result<UserMapping, ()> {
    let code = allocator.allocate_frame().ok_or(())?;
    let stack = allocator.allocate_frame().ok_or(())?;
    let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
    let code_page = Page::containing_address(VirtAddr::new(USER_CODE_ADDRESS));
    let stack_page = Page::containing_address(VirtAddr::new(USER_STACK_PAGE));
    let code_flush = unsafe { mapper.map_to(code_page, code, flags, allocator) }.map_err(|_| ())?;
    let stack_flush = unsafe {
        mapper.map_to(
            stack_page,
            stack,
            flags | PageTableFlags::WRITABLE,
            allocator,
        )
    }
    .map_err(|_| ())?;
    code_flush.flush();
    stack_flush.flush();
    Ok(UserMapping { code, stack })
}

pub fn user_mapping_smoke_test(mapper: &OffsetPageTable<'static>, _mapping: UserMapping) -> bool {
    let code = mapper
        .translate_addr(VirtAddr::new(USER_CODE_ADDRESS))
        .is_some();
    let stack = mapper
        .translate_addr(VirtAddr::new(USER_STACK_PAGE))
        .is_some();
    let guard = mapper
        .translate_addr(VirtAddr::new(USER_GUARD_PAGE))
        .is_none();
    code && stack && guard
}

pub fn validate_user_read(pointer: u64, length: u64) -> bool {
    valid_user_range(pointer, length)
}

pub fn validate_user_write(pointer: u64, length: u64) -> bool {
    valid_user_range(pointer, length)
}

fn valid_user_range(pointer: u64, length: u64) -> bool {
    let Some(end) = pointer.checked_add(length) else {
        return false;
    };
    let code_range = pointer >= USER_CODE_ADDRESS && end <= USER_GUARD_PAGE;
    let stack_range = pointer >= USER_STACK_PAGE && end <= USER_STACK_TOP;
    code_range || stack_range
}

pub fn userspace_validation_smoke_test() -> bool {
    validate_user_read(USER_CODE_ADDRESS, 16)
        && validate_user_write(USER_STACK_PAGE, 16)
        && !validate_user_read(USER_GUARD_PAGE, 1)
        && !validate_user_read(u64::MAX - 4, 16)
}

#[derive(Clone, Copy, Debug)]
pub struct ElfSegment {
    pub offset: usize,
    pub vaddr: u64,
    pub file_size: usize,
    pub mem_size: usize,
    pub flags: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct LoadReport {
    pub segments_loaded: usize,
    pub bytes_loaded: usize,
}

#[derive(Debug)]
pub enum ElfError {
    TooSmall,
    BadMagic,
    UnsupportedClass,
    UnsupportedMachine,
    InvalidHeader,
    InvalidProgramHeader,
    NoLoadableSegment,
}

pub struct ElfImage<'a> {
    bytes: &'a [u8],
    entry: u64,
    segments: Vec<ElfSegment>,
}

impl<'a> ElfImage<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, ElfError> {
        if bytes.len() < 64 {
            return Err(ElfError::TooSmall);
        }
        if &bytes[0..4] != b"\x7fELF" {
            return Err(ElfError::BadMagic);
        }
        if bytes[4] != 2 || bytes[5] != 1 {
            return Err(ElfError::UnsupportedClass);
        }
        if read_u16(bytes, 18) != Some(0x3E) {
            return Err(ElfError::UnsupportedMachine);
        }
        let entry = read_u64(bytes, 24).ok_or(ElfError::InvalidHeader)?;
        let phoff = read_u64(bytes, 32).ok_or(ElfError::InvalidHeader)? as usize;
        let phentsize = read_u16(bytes, 54).ok_or(ElfError::InvalidHeader)? as usize;
        let phnum = read_u16(bytes, 56).ok_or(ElfError::InvalidHeader)? as usize;
        if phentsize < 56 || phnum == 0 || phnum > 32 {
            return Err(ElfError::InvalidHeader);
        }
        let mut segments = Vec::new();
        for index in 0..phnum {
            let Some(position) = phoff.checked_add(index.saturating_mul(phentsize)) else {
                return Err(ElfError::InvalidProgramHeader);
            };
            if position.checked_add(56).is_none_or(|end| end > bytes.len()) {
                return Err(ElfError::InvalidProgramHeader);
            }
            if read_u32(bytes, position) != Some(1) {
                continue;
            }
            let flags = read_u32(bytes, position + 4).ok_or(ElfError::InvalidProgramHeader)?;
            let offset =
                read_u64(bytes, position + 8).ok_or(ElfError::InvalidProgramHeader)? as usize;
            let vaddr = read_u64(bytes, position + 16).ok_or(ElfError::InvalidProgramHeader)?;
            let file_size =
                read_u64(bytes, position + 32).ok_or(ElfError::InvalidProgramHeader)? as usize;
            let mem_size =
                read_u64(bytes, position + 40).ok_or(ElfError::InvalidProgramHeader)? as usize;
            if mem_size < file_size
                || offset
                    .checked_add(file_size)
                    .is_none_or(|end| end > bytes.len())
                || vaddr < USER_CODE_ADDRESS
                || vaddr >= USER_STACK_PAGE
            {
                return Err(ElfError::InvalidProgramHeader);
            }
            segments.push(ElfSegment {
                offset,
                vaddr,
                file_size,
                mem_size,
                flags,
            });
        }
        if segments.is_empty() {
            return Err(ElfError::NoLoadableSegment);
        }
        Ok(Self {
            bytes,
            entry,
            segments,
        })
    }

    pub fn entry(&self) -> u64 {
        self.entry
    }
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
    pub fn segments(&self) -> &[ElfSegment] {
        &self.segments
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}
fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}
fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}

pub fn elf_parser_smoke_test() -> bool {
    let valid = crate::fs::ramdisk_init_elf();
    let good = ElfImage::parse(valid).is_ok();
    let bad = ElfImage::parse(b"not an elf").is_err();
    good && bad
}

pub unsafe fn enter_user(entry: VirtAddr, stack: VirtAddr, selectors: crate::gdt::Selectors) -> ! {
    let entry = entry.as_u64();
    let stack = stack.as_u64();
    let flags = x86_64::registers::rflags::read().bits() | (1 << 9);
    let code = selectors.user_code.0 as u64;
    let data = selectors.user_data.0 as u64;
    // SAFETY: caller supplies mapped user entry/stack and Ring3 selectors;
    // the five pushes exactly form the architectural iretq frame.
    unsafe {
        asm!(
            "push {data}",
            "push {stack}",
            "push {flags}",
            "push {code}",
            "push {entry}",
            "iretq",
            data = in(reg) data,
            stack = in(reg) stack,
            flags = in(reg) flags,
            code = in(reg) code,
            entry = in(reg) entry,
            options(noreturn)
        );
    }
}
