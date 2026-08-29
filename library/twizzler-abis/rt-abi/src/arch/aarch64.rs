#[cfg(feature = "rt0")]
#[no_mangle]
/// Entry for upcalls.
///
/// # Safety
/// This function may not be called except as an upcall from the kernel.
pub unsafe extern "C-unwind" fn __twz_rt_upcall_entry(
    _rdi: *mut core::ffi::c_void,
    _rsi: *const core::ffi::c_void,
) -> ! {
    todo!()
}
