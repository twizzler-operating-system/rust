#[cfg(feature = "rt0")]
#[no_mangle]
/// Entry for upcalls.
///
/// # Safety
/// This function may not be called except as an upcall from the kernel.
pub unsafe extern "C-unwind" fn __twz_rt_upcall_entry(
    rdi: *mut core::ffi::c_void,
    rsi: *const core::ffi::c_void,
) -> ! {
    core::arch::asm!(
        "and rsp, 0xfffffffffffffff0",
        "mov rbp, rdx",
        "push 0",
        "jmp twz_rt_upcall_entry_c",
        in("rdi") rdi,
        in("rsi") rsi,
        options(noreturn)
    );
}
