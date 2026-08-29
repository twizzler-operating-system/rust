//! Functions for allocating memory from the runtime.

use core::alloc::Layout;

use crate::nk;

bitflags::bitflags! {
    /// Flags for allocation functions.
    pub struct AllocFlags : crate::bindings::alloc_flags {
        /// Zero memory. The memory zeroed depends on the function called.
        const ZERO_MEMORY = crate::bindings::ZERO_MEMORY;
    }
}

/// Allocate runtime memory with the given layout and flags. Returns None on allocation failure.
///
/// # Flags
///   - ZERO_MEMORY: Zero the newly-allocated memory before returning it to the user.
pub fn twz_rt_malloc(layout: Layout, flags: AllocFlags) -> Option<*mut u8> {
    unsafe {
        let ptr = nk!(crate::bindings::twz_rt_malloc(
            layout.size(),
            layout.align(),
            flags.bits()
        ));
        if ptr.is_null() {
            None
        } else {
            Some(ptr.cast())
        }
    }
}

/// Reallocate runtime memory pointed to by ptr, with a given layout and flags, to new_size. The new
/// size can be larger or smaller than the original, and may move while maintaining layout
/// constraints. If the allocation moves, the old memory will be copied over and automatically
/// freed.
///
/// # Safety
/// Caller must ensure that any no other references to this memory are alive.
///
/// # Flags
///   - ZERO_MEMORY: If the allocation size grows, zero the newly-allocated memory before returning
///     it to the user. If the allocation size shrinks, zero the old, now unused, part of the memory
///     before freeing. If the allocation moves, zero the old allocation before freeing.
pub unsafe fn twz_rt_realloc(
    ptr: *mut u8,
    layout: Layout,
    new_size: usize,
    flags: AllocFlags,
) -> Option<*mut u8> {
    unsafe {
        let ptr = nk!(crate::bindings::twz_rt_realloc(
            ptr.cast(),
            layout.size(),
            layout.align(),
            new_size,
            flags.bits(),
        ));
        if ptr.is_null() {
            None
        } else {
            Some(ptr.cast())
        }
    }
}

/// Deallocate runtime memory pointed to by ptr, with a given layout and flags.
///
/// # Safety
/// Caller must ensure that any no other references to this memory are alive.
///
/// # Flags
///   - ZERO_MEMORY: Zero the old memory before freeing.
pub unsafe fn twz_rt_dealloc(ptr: *mut u8, layout: Layout, flags: AllocFlags) {
    unsafe {
        nk!(crate::bindings::twz_rt_dealloc(
            ptr.cast(),
            layout.size(),
            layout.align(),
            flags.bits()
        ))
    }
}
