use bootloader_api::info::{MemoryRegion, MemoryRegions};
use x86_64::VirtAddr;

const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";
const SDT_HEADER_LEN: usize = 36;
const RSDP_V1_LEN: usize = 20;
const RSDP_V2_LEN: usize = 36;
const MAX_ACPI_TABLE: usize = 4096;
const MAX_DISCOVERED_CPUS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcpiError {
    MissingRsdp,
    AddressOverflow,
    UnmappedPhysicalRange,
    ShortTable,
    BadSignature,
    BadChecksum,
    InvalidLength,
    InvalidRootEntries,
    MissingMadt,
    MalformedMadt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RootInfo {
    address: u64,
    xsdt: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MadtInfo {
    local_apic_address: u32,
    processors: u32,
    cpu_ids: [u32; MAX_DISCOVERED_CPUS],
    ioapics: u32,
    ioapic_address: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcpiReport {
    pub rsdp: bool,
    pub xsdt: bool,
    pub madt: bool,
    pub cpus: u32,
    pub cpu_ids: [u32; MAX_DISCOVERED_CPUS],
    pub ioapics: u32,
    pub ioapic_address: u32,
    pub local_apic_address: u32,
}

impl AcpiReport {
    const fn unavailable() -> Self {
        Self {
            rsdp: false,
            xsdt: false,
            madt: false,
            cpus: 0,
            cpu_ids: [0; MAX_DISCOVERED_CPUS],
            ioapics: 0,
            ioapic_address: 0,
            local_apic_address: 0,
        }
    }
}

struct PhysicalReader<'a> {
    offset: VirtAddr,
    regions: &'a [MemoryRegion],
}

impl<'a> PhysicalReader<'a> {
    const fn new(offset: VirtAddr, regions: &'a [MemoryRegion]) -> Self {
        Self { offset, regions }
    }

    fn read(&self, physical: u64, output: &mut [u8]) -> Result<(), AcpiError> {
        if output.is_empty() {
            return Ok(());
        }
        let length = output.len() as u64;
        let end = physical
            .checked_add(length)
            .ok_or(AcpiError::AddressOverflow)?;
        let virtual_address = self
            .offset
            .as_u64()
            .checked_add(physical)
            .ok_or(AcpiError::AddressOverflow)?;
        let described = self
            .regions
            .iter()
            .any(|region| physical >= region.start && end <= region.end);
        if !described {
            return Err(AcpiError::UnmappedPhysicalRange);
        }

        // The bootloader's physical-memory mapping is established before the
        // kernel entry point and covers every memory region it reports. The
        // range and address-overflow checks above make this copy bounded; ACPI
        // contents remain untrusted firmware data and are validated separately.
        unsafe {
            core::ptr::copy_nonoverlapping(
                virtual_address as *const u8,
                output.as_mut_ptr(),
                output.len(),
            );
        }
        Ok(())
    }
}

pub fn discover(
    rsdp_address: Option<u64>,
    physical_memory_offset: VirtAddr,
    memory_regions: &MemoryRegions,
) -> AcpiReport {
    let Some(rsdp_address) = rsdp_address else {
        return AcpiReport::unavailable();
    };
    let reader = PhysicalReader::new(physical_memory_offset, memory_regions);
    match discover_inner(&reader, rsdp_address) {
        Ok(report) => report,
        Err(_) => AcpiReport::unavailable(),
    }
}

fn discover_inner(reader: &PhysicalReader<'_>, rsdp_address: u64) -> Result<AcpiReport, AcpiError> {
    let mut rsdp = [0u8; RSDP_V2_LEN];
    reader.read(rsdp_address, &mut rsdp)?;
    let root = parse_rsdp(&rsdp)?;

    let mut root_header = [0u8; SDT_HEADER_LEN];
    reader.read(root.address, &mut root_header)?;
    let root_length = checked_table_length(&root_header)?;
    let expected_signature = if root.xsdt { b"XSDT" } else { b"RSDT" };
    if &root_header[0..4] != expected_signature {
        return Err(AcpiError::BadSignature);
    }
    let mut root_table = [0u8; MAX_ACPI_TABLE];
    reader.read(root.address, &mut root_table[..root_length])?;
    if !checksum_valid(&root_table[..root_length]) {
        return Err(AcpiError::BadChecksum);
    }

    let entry_size = if root.xsdt { 8 } else { 4 };
    let entries_len = root_length
        .checked_sub(SDT_HEADER_LEN)
        .ok_or(AcpiError::InvalidLength)?;
    if entries_len % entry_size != 0 {
        return Err(AcpiError::InvalidRootEntries);
    }

    let mut madt = None;
    for entry in root_table[SDT_HEADER_LEN..root_length].chunks_exact(entry_size) {
        let address = if root.xsdt {
            u64::from_le_bytes(
                entry
                    .try_into()
                    .map_err(|_| AcpiError::InvalidRootEntries)?,
            )
        } else {
            u32::from_le_bytes(
                entry
                    .try_into()
                    .map_err(|_| AcpiError::InvalidRootEntries)?,
            ) as u64
        };
        if address == 0 {
            continue;
        }
        let mut header = [0u8; SDT_HEADER_LEN];
        reader.read(address, &mut header)?;
        if &header[0..4] != b"APIC" {
            continue;
        }
        let length = checked_table_length(&header)?;
        let mut table = [0u8; MAX_ACPI_TABLE];
        reader.read(address, &mut table[..length])?;
        madt = Some(parse_madt(&table[..length])?);
        break;
    }

    let madt = madt.ok_or(AcpiError::MissingMadt)?;
    Ok(AcpiReport {
        rsdp: true,
        xsdt: root.xsdt,
        madt: true,
        cpus: madt.processors,
        cpu_ids: madt.cpu_ids,
        ioapics: madt.ioapics,
        ioapic_address: madt.ioapic_address,
        local_apic_address: madt.local_apic_address,
    })
}

fn parse_rsdp(bytes: &[u8; RSDP_V2_LEN]) -> Result<RootInfo, AcpiError> {
    if &bytes[0..8] != RSDP_SIGNATURE {
        return Err(AcpiError::BadSignature);
    }
    if !checksum_valid(&bytes[..RSDP_V1_LEN]) {
        return Err(AcpiError::BadChecksum);
    }
    let revision = bytes[15];
    let (address, xsdt) = if revision >= 2 {
        let length = u32::from_le_bytes(bytes[20..24].try_into().unwrap()) as usize;
        if length < RSDP_V2_LEN || length > MAX_ACPI_TABLE {
            return Err(AcpiError::InvalidLength);
        }
        if length > bytes.len() || !checksum_valid(&bytes[..length]) {
            return Err(AcpiError::BadChecksum);
        }
        (u64::from_le_bytes(bytes[24..32].try_into().unwrap()), true)
    } else {
        (
            u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as u64,
            false,
        )
    };
    if address == 0 {
        return Err(AcpiError::InvalidLength);
    }
    Ok(RootInfo { address, xsdt })
}

fn checked_table_length(header: &[u8; SDT_HEADER_LEN]) -> Result<usize, AcpiError> {
    let length = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    if !(SDT_HEADER_LEN..=MAX_ACPI_TABLE).contains(&length) {
        return Err(AcpiError::InvalidLength);
    }
    Ok(length)
}

fn parse_madt(table: &[u8]) -> Result<MadtInfo, AcpiError> {
    if table.len() < SDT_HEADER_LEN + 8 || &table[0..4] != b"APIC" {
        return Err(AcpiError::MalformedMadt);
    }
    let declared_length = u32::from_le_bytes(table[4..8].try_into().unwrap()) as usize;
    if declared_length != table.len() || !checksum_valid(table) {
        return Err(AcpiError::BadChecksum);
    }

    let mut local_apic_address = u32::from_le_bytes(table[36..40].try_into().unwrap());
    let mut processors = 0;
    let mut cpu_ids = [0; MAX_DISCOVERED_CPUS];
    let mut ioapics = 0;
    let mut ioapic_address = 0;
    let mut cursor = 44;
    while cursor < table.len() {
        if table.len() - cursor < 2 {
            return Err(AcpiError::MalformedMadt);
        }
        let entry_type = table[cursor];
        let entry_length = table[cursor + 1] as usize;
        if entry_length < 2 || entry_length > table.len() - cursor {
            return Err(AcpiError::MalformedMadt);
        }
        let entry = &table[cursor..cursor + entry_length];
        match entry_type {
            0 if entry_length >= 8 => {
                let flags = u32::from_le_bytes(entry[4..8].try_into().unwrap());
                if flags & 1 != 0 {
                    if (processors as usize) < cpu_ids.len() {
                        cpu_ids[processors as usize] = entry[3] as u32;
                        processors += 1;
                    }
                }
            }
            1 if entry_length >= 12 => {
                ioapics += 1;
                if ioapic_address == 0 {
                    ioapic_address = u32::from_le_bytes(entry[4..8].try_into().unwrap());
                }
            }
            5 if entry_length >= 12 => {
                local_apic_address = u32::from_le_bytes(entry[8..12].try_into().unwrap());
            }
            9 if entry_length >= 16 => {
                let flags = u32::from_le_bytes(entry[8..12].try_into().unwrap());
                if flags & 1 != 0 {
                    if (processors as usize) < cpu_ids.len() {
                        cpu_ids[processors as usize] =
                            u32::from_le_bytes(entry[4..8].try_into().unwrap());
                        processors += 1;
                    }
                }
            }
            _ => {}
        }
        cursor += entry_length;
    }
    Ok(MadtInfo {
        local_apic_address,
        processors,
        cpu_ids,
        ioapics,
        ioapic_address,
    })
}

fn checksum_valid(bytes: &[u8]) -> bool {
    bytes.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checksum(bytes: &mut [u8]) {
        let sum = bytes[..bytes.len() - 1]
            .iter()
            .fold(0u8, |sum, byte| sum.wrapping_add(*byte));
        let last = bytes.len() - 1;
        bytes[last] = 0u8.wrapping_sub(sum);
    }

    #[test]
    fn rejects_bad_rsdp_signature_and_checksum() {
        let mut rsdp = [0u8; RSDP_V2_LEN];
        assert_eq!(parse_rsdp(&rsdp), Err(AcpiError::BadSignature));
        rsdp[0..8].copy_from_slice(RSDP_SIGNATURE);
        assert_eq!(parse_rsdp(&rsdp), Err(AcpiError::BadChecksum));
    }

    #[test]
    fn accepts_valid_xsdt_rsdp() {
        let mut rsdp = [0u8; RSDP_V2_LEN];
        rsdp[0..8].copy_from_slice(RSDP_SIGNATURE);
        rsdp[15] = 2;
        rsdp[20..24].copy_from_slice(&(RSDP_V2_LEN as u32).to_le_bytes());
        rsdp[24..32].copy_from_slice(&0x1234u64.to_le_bytes());
        checksum(&mut rsdp[0..20]);
        checksum(&mut rsdp);
        assert_eq!(
            parse_rsdp(&rsdp),
            Ok(RootInfo {
                address: 0x1234,
                xsdt: true
            })
        );
    }

    #[test]
    fn rejects_malformed_madt_entry() {
        let mut madt = [0u8; 52];
        madt[0..4].copy_from_slice(b"APIC");
        let madt_len = madt.len() as u32;
        madt[4..8].copy_from_slice(&madt_len.to_le_bytes());
        madt[40..44].copy_from_slice(&1u32.to_le_bytes());
        madt[44] = 0;
        madt[45] = 1;
        checksum(&mut madt);
        assert_eq!(parse_madt(&madt), Err(AcpiError::MalformedMadt));
    }

    #[test]
    fn parses_enabled_cpu_and_ioapic() {
        let mut madt = [0u8; 64];
        madt[0..4].copy_from_slice(b"APIC");
        let madt_len = madt.len() as u32;
        madt[4..8].copy_from_slice(&madt_len.to_le_bytes());
        madt[36..40].copy_from_slice(&0xfee00000u32.to_le_bytes());
        madt[44] = 0;
        madt[45] = 8;
        madt[48] = 1;
        madt[52] = 1;
        madt[53] = 12;
        checksum(&mut madt);
        let parsed = parse_madt(&madt).unwrap();
        assert_eq!(parsed.processors, 1);
        assert_eq!(parsed.ioapics, 1);
        assert_eq!(parsed.ioapic_address, 0);
    }
}
