use core::arch::asm;

const ATA_DATA: u16 = 0x1F0;
const ATA_SECTOR_COUNT: u16 = 0x1F2;
const ATA_LBA_LOW: u16 = 0x1F3;
const ATA_LBA_MID: u16 = 0x1F4;
const ATA_LBA_HIGH: u16 = 0x1F5;
const ATA_DRIVE: u16 = 0x1F6;
const ATA_STATUS: u16 = 0x1F7;
const ATA_COMMAND: u16 = 0x1F7;
const ATA_READ_SECTORS: u8 = 0x20;
const ATA_WRITE_SECTORS: u8 = 0x30;
const STATUS_BSY: u8 = 0x80;
const STATUS_DRQ: u8 = 0x08;
const STATUS_ERR: u8 = 0x01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartitionEntry {
    pub bootable: bool,
    pub partition_type: u8,
    pub start_lba: u32,
    pub sectors: u32,
}

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags))
    }
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags))
    }
    value
}

unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    unsafe {
        asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack, preserves_flags))
    }
    value
}

unsafe fn outw(port: u16, value: u16) {
    unsafe {
        asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags))
    }
}

pub fn read_sector(lba: u32, output: &mut [u8; 512]) -> bool {
    unsafe {
        outb(ATA_DRIVE, 0xE0 | ((lba >> 24) as u8 & 0x0F));
        outb(ATA_SECTOR_COUNT, 1);
        outb(ATA_LBA_LOW, lba as u8);
        outb(ATA_LBA_MID, (lba >> 8) as u8);
        outb(ATA_LBA_HIGH, (lba >> 16) as u8);
        outb(ATA_COMMAND, ATA_READ_SECTORS);
        let mut ready = false;
        for _ in 0..1_000_000 {
            let status = inb(ATA_STATUS);
            if status & STATUS_ERR != 0 {
                return false;
            }
            if status & STATUS_BSY == 0 && status & STATUS_DRQ != 0 {
                ready = true;
                break;
            }
            core::hint::spin_loop();
        }
        if !ready {
            return false;
        }
        for chunk in output.chunks_exact_mut(2) {
            let word = inw(ATA_DATA).to_le_bytes();
            chunk.copy_from_slice(&word);
        }
    }
    true
}

pub fn write_sector(lba: u32, input: &[u8; 512]) -> bool {
    unsafe {
        outb(ATA_DRIVE, 0xE0 | ((lba >> 24) as u8 & 0x0F));
        outb(ATA_SECTOR_COUNT, 1);
        outb(ATA_LBA_LOW, lba as u8);
        outb(ATA_LBA_MID, (lba >> 8) as u8);
        outb(ATA_LBA_HIGH, (lba >> 16) as u8);
        outb(ATA_COMMAND, ATA_WRITE_SECTORS);
        let mut ready = false;
        for _ in 0..1_000_000 {
            let status = inb(ATA_STATUS);
            if status & STATUS_ERR != 0 {
                return false;
            }
            if status & STATUS_BSY == 0 && status & STATUS_DRQ != 0 {
                ready = true;
                break;
            }
            core::hint::spin_loop();
        }
        if !ready {
            return false;
        }
        for chunk in input.chunks_exact(2) {
            outw(ATA_DATA, u16::from_le_bytes([chunk[0], chunk[1]]));
        }
        for _ in 0..1_000_000 {
            let status = inb(ATA_STATUS);
            if status & STATUS_ERR != 0 {
                return false;
            }
            if status & STATUS_BSY == 0 {
                return true;
            }
            core::hint::spin_loop();
        }
    }
    false
}

pub fn read_mbr_partitions() -> Option<[PartitionEntry; 4]> {
    let mut sector = [0_u8; 512];
    if !read_sector(0, &mut sector) || sector[510] != 0x55 || sector[511] != 0xAA {
        return None;
    }
    let mut entries = [PartitionEntry {
        bootable: false,
        partition_type: 0,
        start_lba: 0,
        sectors: 0,
    }; 4];
    for (index, entry) in entries.iter_mut().enumerate() {
        let offset = 446 + index * 16;
        entry.bootable = sector[offset] == 0x80;
        entry.partition_type = sector[offset + 4];
        entry.start_lba = u32::from_le_bytes(sector[offset + 8..offset + 12].try_into().ok()?);
        entry.sectors = u32::from_le_bytes(sector[offset + 12..offset + 16].try_into().ok()?);
    }
    Some(entries)
}

pub fn disk_smoke_test() -> bool {
    let Some(partitions) = read_mbr_partitions() else {
        return false;
    };
    partitions
        .iter()
        .any(|entry| entry.partition_type != 0 && entry.sectors != 0)
}
