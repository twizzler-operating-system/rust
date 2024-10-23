use crate::alloc::{GlobalAlloc, Layout, System};
use twizzler_rt_abi::alloc::AllocFlags;

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        twizzler_rt_abi::alloc::twz_rt_malloc(layout, AllocFlags::empty()).expect("failed to allocate memory")
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        twizzler_rt_abi::alloc::twz_rt_malloc(layout, AllocFlags::ZERO_MEMORY).expect("failed to allocate memory")
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        twizzler_rt_abi::alloc::twz_rt_dealloc(ptr, layout, AllocFlags::empty());
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        twizzler_rt_abi::alloc::twz_rt_realloc(ptr, layout, new_size, AllocFlags::empty()).expect("failed to reallocate memory")
    }
}
