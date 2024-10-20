use crate::alloc::{GlobalAlloc, Layout, System};

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let runtime = twizzler_runtime_api::get_runtime();
        unsafe { runtime.default_allocator().alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let runtime = twizzler_runtime_api::get_runtime();
        unsafe { runtime.default_allocator().alloc_zeroed(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let runtime = twizzler_runtime_api::get_runtime();
        unsafe { runtime.default_allocator().dealloc(ptr, layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let runtime = twizzler_runtime_api::get_runtime();
        unsafe { runtime.default_allocator().realloc(ptr, layout, new_size) }
    }
}
