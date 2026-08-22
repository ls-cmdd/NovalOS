use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use x86_64::VirtAddr;

use crate::acpi::AcpiReport;

const APIC_BASE_MSR: u32 = 0x1b;
const APIC_BASE_MASK: u64 = 0x000f_ffff_f000;
const APIC_GLOBAL_ENABLE: u64 = 1 << 11;

const LAPIC_ID: u64 = 0x020;
const LAPIC_EOI: u64 = 0x0b0;
const LAPIC_SPURIOUS: u64 = 0x0f0;
const LAPIC_LVT_TIMER: u64 = 0x320;
const LAPIC_TIMER_INITIAL: u64 = 0x380;
const LAPIC_ICR_LOW: u64 = 0x300;
const LAPIC_ICR_HIGH: u64 = 0x310;
const LAPIC_TIMER_DIVIDE: u64 = 0x3e0;
const LAPIC_TIMER_PERIODIC: u32 = 1 << 17;
const LAPIC_TIMER_VECTOR: u32 = 32;
const LAPIC_SPURIOUS_VECTOR: u32 = 0xff;

const IOAPIC_REGSEL: u64 = 0x00;
const IOAPIC_WINDOW: u64 = 0x10;
const IOAPIC_VERSION: u8 = 1;
const IOAPIC_KEYBOARD_IRQ: u8 = 1;
const IOAPIC_KEYBOARD_VECTOR: u8 = 33;

static LAPIC_ENABLED: AtomicBool = AtomicBool::new(false);
static LAPIC_VIRTUAL_BASE: AtomicU64 = AtomicU64::new(0);
static IOAPIC_VIRTUAL_BASE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApicInfo {
    pub has_xapic: bool,
    pub has_x2apic: bool,
    pub initial_local_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutingReport {
    pub lapic_enabled: bool,
    pub ioapic_enabled: bool,
    pub timer_vector: u8,
    pub keyboard_vector: u8,
    pub local_id: u32,
}

pub fn detect() -> ApicInfo {
    #[cfg(target_arch = "x86_64")]
    {
        // CPUID.1:ECX bit 21 is x2APIC; EDX bit 9 is xAPIC.
        let leaf = unsafe { core::arch::x86_64::__cpuid(1) };
        let has_xapic = leaf.edx & (1 << 9) != 0;
        let has_x2apic = leaf.ecx & (1 << 21) != 0;
        return ApicInfo {
            has_xapic,
            has_x2apic,
            initial_local_id: 0,
        };
    }
    #[allow(unreachable_code)]
    ApicInfo {
        has_xapic: false,
        has_x2apic: false,
        initial_local_id: 0,
    }
}

pub fn init(report: AcpiReport, physical_memory_offset: VirtAddr) -> RoutingReport {
    if !report.madt || report.local_apic_address == 0 || report.ioapic_address == 0 {
        return RoutingReport {
            lapic_enabled: false,
            ioapic_enabled: false,
            timer_vector: LAPIC_TIMER_VECTOR as u8,
            keyboard_vector: IOAPIC_KEYBOARD_VECTOR,
            local_id: 0,
        };
    }

    let current_base = read_msr(APIC_BASE_MSR);
    let lapic_physical = (current_base & APIC_BASE_MASK).max(report.local_apic_address as u64);
    let lapic_virtual = physical_memory_offset
        .as_u64()
        .checked_add(lapic_physical)
        .unwrap_or(0);
    let ioapic_virtual = physical_memory_offset
        .as_u64()
        .checked_add(report.ioapic_address as u64)
        .unwrap_or(0);
    if lapic_virtual == 0 || ioapic_virtual == 0 {
        return RoutingReport {
            lapic_enabled: false,
            ioapic_enabled: false,
            timer_vector: LAPIC_TIMER_VECTOR as u8,
            keyboard_vector: IOAPIC_KEYBOARD_VECTOR,
            local_id: 0,
        };
    }

    write_msr(
        APIC_BASE_MSR,
        (current_base & APIC_BASE_MASK) | APIC_GLOBAL_ENABLE,
    );
    LAPIC_VIRTUAL_BASE.store(lapic_virtual, Ordering::Release);
    IOAPIC_VIRTUAL_BASE.store(ioapic_virtual, Ordering::Release);

    lapic_write(LAPIC_SPURIOUS, LAPIC_SPURIOUS_VECTOR | (1 << 8));
    lapic_write(LAPIC_LVT_TIMER, LAPIC_TIMER_VECTOR | LAPIC_TIMER_PERIODIC);
    lapic_write(LAPIC_TIMER_DIVIDE, 0b0011);
    lapic_write(LAPIC_TIMER_INITIAL, 1_000_000);

    let local_id = lapic_read(LAPIC_ID) >> 24;
    let ioapic_enabled = configure_ioapic_keyboard(local_id as u8);
    let lapic_enabled = true;
    LAPIC_ENABLED.store(lapic_enabled && ioapic_enabled, Ordering::Release);

    RoutingReport {
        lapic_enabled,
        ioapic_enabled,
        timer_vector: LAPIC_TIMER_VECTOR as u8,
        keyboard_vector: IOAPIC_KEYBOARD_VECTOR,
        local_id,
    }
}

pub fn enabled() -> bool {
    LAPIC_ENABLED.load(Ordering::Acquire)
}

pub fn start_ap(local_id: u32, startup_vector: u8) -> bool {
    if !enabled() || local_id > 0xff {
        return false;
    }
    lapic_write(LAPIC_ICR_HIGH, local_id << 24);
    lapic_write(LAPIC_ICR_LOW, 0x0000_4500); // INIT assert, level-triggered
    let _ = wait_for_icr_idle();
    lapic_write(LAPIC_ICR_LOW, 0x0000_0500); // INIT deassert
    let _ = wait_for_icr_idle();
    for _ in 0..100_000 {
        core::hint::spin_loop();
    }
    let sipi = 0x0000_4600 | startup_vector as u32;
    lapic_write(LAPIC_ICR_HIGH, local_id << 24);
    lapic_write(LAPIC_ICR_LOW, sipi);
    let _ = wait_for_icr_idle();
    for _ in 0..200_000 {
        core::hint::spin_loop();
    }
    // A second SIPI is required by the MP startup protocol for APs that did
    // not observe the first wakeup while leaving INIT state.
    lapic_write(LAPIC_ICR_HIGH, local_id << 24);
    lapic_write(LAPIC_ICR_LOW, sipi);
    let _ = wait_for_icr_idle();
    true
}

fn wait_for_icr_idle() -> bool {
    for _ in 0..1_000_000 {
        if lapic_read(LAPIC_ICR_LOW) & 1 == 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

pub fn end_of_interrupt() {
    if enabled() {
        lapic_write(LAPIC_EOI, 0);
    } else {
        crate::pic::send_eoi(0);
    }
}

fn configure_ioapic_keyboard(destination: u8) -> bool {
    let ioapic = IOAPIC_VIRTUAL_BASE.load(Ordering::Acquire);
    if ioapic == 0 {
        return false;
    }
    let version = ioapic_read(ioapic, IOAPIC_VERSION);
    let max_redirection = ((version >> 16) & 0xff) as u8;
    if max_redirection < IOAPIC_KEYBOARD_IRQ {
        return false;
    }

    let low = IOAPIC_KEYBOARD_VECTOR as u32;
    let high = (destination as u32) << 24;
    let redirection_index = 0x10 + (IOAPIC_KEYBOARD_IRQ as u8 * 2);
    ioapic_write(ioapic, redirection_index, low);
    ioapic_write(ioapic, redirection_index + 1, high);
    true
}

fn lapic_read(register: u64) -> u32 {
    let base = LAPIC_VIRTUAL_BASE.load(Ordering::Acquire);
    unsafe { core::ptr::read_volatile((base + register) as *const u32) }
}

fn lapic_write(register: u64, value: u32) {
    let base = LAPIC_VIRTUAL_BASE.load(Ordering::Acquire);
    unsafe { core::ptr::write_volatile((base + register) as *mut u32, value) };
}

fn ioapic_read(base: u64, register: u8) -> u32 {
    unsafe {
        core::ptr::write_volatile((base + IOAPIC_REGSEL) as *mut u32, register as u32);
        core::ptr::read_volatile((base + IOAPIC_WINDOW) as *const u32)
    }
}

fn ioapic_write(base: u64, register: u8, value: u32) {
    unsafe {
        core::ptr::write_volatile((base + IOAPIC_REGSEL) as *mut u32, register as u32);
        core::ptr::write_volatile((base + IOAPIC_WINDOW) as *mut u32, value);
    }
}

fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((high as u64) << 32) | low as u64
}

fn write_msr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
}
