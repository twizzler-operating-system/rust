use crate::os::raw::c_char;

pub mod args;
pub mod env;
pub mod fd;
pub mod fs;
pub mod futex;
pub mod io;
pub mod net;
pub mod os;
#[path = "../unsupported/pipe.rs"]
pub mod pipe;
#[path = "../unsupported/process.rs"]
pub mod process;
#[path = "../unix/stdio.rs"]
pub mod stdio;
pub mod thread;
pub mod time;

pub fn unsupported<T>() -> crate::io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> crate::io::Error {
    crate::io::const_io_error!(
        crate::io::ErrorKind::Unsupported,
        &"operation not supported on Twizzler yet",
    )
}

#[inline]
pub fn abort_internal() -> ! {
    twizzler_rt_abi::core::twz_rt_abort()
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[no_mangle]
// NB. used by both libunwind and libpanic_abort
pub extern "C" fn __rust_abort() {
    abort_internal();
}

// SAFETY: must be called only once during runtime initialization.
// NOTE: this is not guaranteed to run, for example when Rust code is called externally.
pub unsafe fn init(argc: isize, argv: *const *const u8, _sigpipe: u8) {
    args::init(argc, argv);
}

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub unsafe fn cleanup() {}

#[inline]
pub(crate) fn is_interrupted(_errno: i32) -> bool {
    false
}

pub fn decode_error_kind(_errno: i32) -> crate::io::ErrorKind {
    crate::io::ErrorKind::Other
}

#[no_mangle]
#[allow(unreachable_code)]
#[allow(unused_variables)]
pub unsafe extern "C" fn std_entry_from_runtime(
    aux: twizzler_rt_abi::core::BasicAux,
) -> twizzler_rt_abi::core::BasicReturn {
    extern "C" {
        fn main(argc: isize, argv: *const *const c_char) -> i32;
    }

    crate::sys::os::init_environment(aux.env as *const *const i8);
    // If pre_main_hook returns a code, then don't call main and exit with that code instead.
    let code = if let Some(pre_code) = twizzler_rt_abi::core::twz_rt_pre_main_hook() {
        pre_code
    } else {
        main(aux.argc as isize, aux.args as *const *const i8)
    };
    twizzler_rt_abi::core::twz_rt_post_main_hook();

    unsafe {
        crate::sys::thread_local::destructors::run();
    }
    crate::rt::thread_cleanup();

    twizzler_rt_abi::core::BasicReturn { code }
}
