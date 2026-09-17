use twizzler_rt_abi::alloc::AllocFlags;

use crate::alloc::{GlobalAlloc, Layout, System};
use crate::ptr;

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    // `GlobalAlloc` requires a null return on failure, not a panic. Panicking here runs the panic
    // machinery *inside* the allocator -- it allocates to format its own message, so the failing
    // allocation is retried from within itself and the process dies as "thread panicked while
    // processing panic" with the real layout never reported. Null lets `handle_alloc_error` do its
    // job, which aborts without allocating and names the size. Every other platform returns null.
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        twizzler_rt_abi::alloc::twz_rt_malloc(layout, AllocFlags::empty())
            .unwrap_or(ptr::null_mut())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        twizzler_rt_abi::alloc::twz_rt_malloc(layout, AllocFlags::ZERO_MEMORY)
            .unwrap_or(ptr::null_mut())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            twizzler_rt_abi::alloc::twz_rt_dealloc(ptr, layout, AllocFlags::empty());
        }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            twizzler_rt_abi::alloc::twz_rt_realloc(ptr, layout, new_size, AllocFlags::empty())
                .unwrap_or(crate::ptr::null_mut())
        }
    }
}
