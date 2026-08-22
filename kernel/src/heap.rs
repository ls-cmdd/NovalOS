use core::alloc::Layout;
#[cfg(not(test))]
use linked_list_allocator::LockedHeap;

const HEAP_SIZE: usize = 128 * 1024;

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[repr(align(16))]
struct HeapStorage([u8; HEAP_SIZE]);

static mut HEAP_STORAGE: HeapStorage = HeapStorage([0; HEAP_SIZE]);

#[cfg(not(test))]
pub unsafe fn init() {
    let start = core::ptr::addr_of_mut!(HEAP_STORAGE.0) as *mut u8;
    // SAFETY: the backing array is static, aligned, and exclusively handed to
    // the allocator exactly once during boot; its mutation is encapsulated by
    // LockedHeap after initialization.
    unsafe { ALLOCATOR.lock().init(start, HEAP_SIZE) };
}

pub fn allocation_smoke_test() -> (usize, u64) {
    let mut values = alloc::vec::Vec::with_capacity(64);
    for value in 0..64_u64 {
        values.push(value.wrapping_mul(17).wrapping_add(3));
    }
    let checksum = values
        .iter()
        .fold(0_u64, |sum, value| sum.wrapping_add(*value));
    (values.capacity(), checksum)
}

#[allow(dead_code)]
pub fn allocation_error_layout(layout: Layout) -> ! {
    panic!("allocation failed: {} bytes", layout.size())
}
