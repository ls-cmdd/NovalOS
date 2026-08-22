use core::arch::asm;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);
static USE_LAPIC: AtomicBool = AtomicBool::new(false);
const PIT_FREQUENCY: u32 = 1_193_182;
const HZ: u32 = 100;

unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags))
    }
}

pub fn init(use_lapic: bool) {
    USE_LAPIC.store(use_lapic, Ordering::Release);
    if !use_lapic {
        let divisor = (PIT_FREQUENCY / HZ) as u16;
        unsafe {
            outb(0x43, 0x36); // channel 0, lobyte/hibyte, mode 3
            outb(0x40, (divisor & 0xFF) as u8);
            outb(0x40, (divisor >> 8) as u8);
        }
    }
    TICKS.store(0, Ordering::Release);
}

pub fn on_interrupt() {
    TICKS.fetch_add(1, Ordering::Relaxed);
    if USE_LAPIC.load(Ordering::Acquire) {
        crate::apic::end_of_interrupt();
    } else {
        crate::pic::send_eoi(0);
    }
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Acquire)
}

pub fn wait_for_ticks(count: u64) {
    let target = ticks().saturating_add(count);
    while ticks() < target {
        x86_64::instructions::hlt();
    }
}
