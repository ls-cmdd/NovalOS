use core::arch::asm;
use spin::Mutex;

const QUEUE_CAPACITY: usize = 64;
struct ScancodeQueue {
    bytes: [u8; QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    len: usize,
}

static QUEUE: Mutex<ScancodeQueue> = Mutex::new(ScancodeQueue {
    bytes: [0; QUEUE_CAPACITY],
    head: 0,
    tail: 0,
    len: 0,
});

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags))
    }
    value
}

pub fn init() {
    let mut queue = QUEUE.lock();
    queue.head = 0;
    queue.tail = 0;
    queue.len = 0;
}

pub fn on_interrupt() {
    let status = unsafe { inb(0x64) };
    if status & 1 == 0 {
        return;
    }
    let scancode = unsafe { inb(0x60) };
    let mut queue = QUEUE.lock();
    if queue.len < QUEUE_CAPACITY {
        let tail = queue.tail;
        queue.bytes[tail] = scancode;
        queue.tail = (tail + 1) % QUEUE_CAPACITY;
        queue.len += 1;
    }
}

pub fn pop_scancode() -> Option<u8> {
    let mut queue = QUEUE.lock();
    if queue.len == 0 {
        return None;
    }
    let head = queue.head;
    let byte = queue.bytes[head];
    queue.head = (head + 1) % QUEUE_CAPACITY;
    queue.len -= 1;
    Some(byte)
}
