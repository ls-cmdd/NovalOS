use crate::block::{self, PartitionEntry};

const NOVELFS_PARTITION_TYPE: u8 = 0xDA;
const SECTOR_SIZE: usize = 512;
const MAGIC: &[u8; 8] = b"NOVELFS1";

#[derive(Clone, Copy)]
struct NovelFs {
    partition: PartitionEntry,
    data_start: u32,
}

impl NovelFs {
    fn mount() -> Option<Self> {
        let partitions = block::read_mbr_partitions()?;
        let partition = partitions
            .into_iter()
            .find(|entry| entry.partition_type == NOVELFS_PARTITION_TYPE && entry.sectors >= 3)?;
        let mut header = [0_u8; SECTOR_SIZE];
        if !block::read_sector(partition.start_lba, &mut header)
            || &header[..MAGIC.len()] != MAGIC
            || u32::from_le_bytes(header[8..12].try_into().ok()?) != 1
            || u32::from_le_bytes(header[12..16].try_into().ok()?) != 1
        {
            return None;
        }
        let data_start = u32::from_le_bytes(header[20..24].try_into().ok()?);
        if data_start >= partition.sectors {
            return None;
        }
        Some(Self {
            partition,
            data_start,
        })
    }

    fn read_file(self, name: &[u8], output: &mut [u8]) -> Option<usize> {
        if name.is_empty() || name.len() > 32 {
            return None;
        }
        let mut directory = [0_u8; SECTOR_SIZE];
        let directory_lba = self.partition.start_lba.checked_add(1)?;
        if !block::read_sector(directory_lba, &mut directory) {
            return None;
        }
        for entry in directory.chunks_exact(40) {
            let name_end = entry.iter().position(|byte| *byte == 0).unwrap_or(32);
            if &entry[..name_end] != name {
                continue;
            }
            let relative_lba = u32::from_le_bytes(entry[32..36].try_into().ok()?);
            let length = u32::from_le_bytes(entry[36..40].try_into().ok()?) as usize;
            let sector_count = length.div_ceil(SECTOR_SIZE);
            let relative_end = relative_lba.checked_add(sector_count as u32)?;
            if relative_lba < self.data_start
                || relative_end > self.partition.sectors
                || length > output.len()
            {
                return None;
            }
            let mut copied = 0;
            for sector_index in 0..sector_count {
                let mut sector = [0_u8; SECTOR_SIZE];
                let lba = self
                    .partition
                    .start_lba
                    .checked_add(relative_lba + sector_index as u32)?;
                if !block::read_sector(lba, &mut sector) {
                    return None;
                }
                let count = (length - copied).min(SECTOR_SIZE);
                output[copied..copied + count].copy_from_slice(&sector[..count]);
                copied += count;
            }
            return Some(length);
        }
        None
    }
}

pub fn persistent_fs_smoke_test() -> bool {
    let Some(fs) = NovelFs::mount() else {
        return false;
    };
    let mut output = [0_u8; 128];
    fs.read_file(b"init.txt", &mut output)
        .is_some_and(|length| output[..length].starts_with(b"NOVEL_OS INIT"))
}

pub fn persistent_fs_rw_smoke_test() -> bool {
    let Some(fs) = NovelFs::mount() else {
        return false;
    };
    let scratch_lba = match fs
        .partition
        .start_lba
        .checked_add(fs.partition.sectors.saturating_sub(1))
    {
        Some(lba) => lba,
        None => return false,
    };
    let mut original = [0_u8; SECTOR_SIZE];
    if !block::read_sector(scratch_lba, &mut original) {
        return false;
    }
    let mut pattern = [0_u8; SECTOR_SIZE];
    let marker = b"NOVELFS-RW-TEST";
    pattern[..marker.len()].copy_from_slice(marker);
    let mut observed = [0_u8; SECTOR_SIZE];
    let wrote = block::write_sector(scratch_lba, &pattern);
    let verified = wrote && block::read_sector(scratch_lba, &mut observed) && observed == pattern;
    let restored = block::write_sector(scratch_lba, &original)
        && block::read_sector(scratch_lba, &mut observed)
        && observed == original;
    verified && restored
}
