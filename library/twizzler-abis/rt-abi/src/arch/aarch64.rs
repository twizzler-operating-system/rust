#[cfg(feature = "rt0")]
#[no_mangle]
/// Entry for upcalls.
///
/// # Safety
/// This function may not be called except as an upcall from the kernel.
pub unsafe extern "C-unwind" fn __twz_rt_upcall_entry(
    frame: *mut core::ffi::c_void,
    info: *const core::ffi::c_void,
) -> ! {
    // The kernel already placed the frame on an aligned stack; the pointers arrive in x0/x1.
    core::arch::asm!(
        "b twz_rt_upcall_entry_c",
        in("x0") frame,
        in("x1") info,
        options(noreturn)
    );
}
