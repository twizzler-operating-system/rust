//! Low-level runtime functionality.

/// Type for exit code.
pub type ExitCode = crate::bindings::exit_code;
use crate::{
    error::{GenericError, TwzError},
    nk,
};

pub fn twz_rt_gc() {
    unsafe {
        nk!(crate::bindings::twz_rt_gc());
    }
}

/// Exit the process with the provided code, from any thread (POSIX `exit()` semantics): all
/// other threads are ended as well.
pub fn twz_rt_exit(code: ExitCode) -> ! {
    unsafe {
        nk!(crate::bindings::twz_rt_exit(code));
        unreachable!()
    }
}

/// Exit only the calling thread. Thread trampolines call this when a thread's entry function
/// returns; nothing else should. Distinct from [`twz_rt_exit`] so the runtime can tell a
/// finished thread from a process-exit request.
pub fn twz_rt_thread_exit(code: ExitCode) -> ! {
    unsafe {
        nk!(crate::bindings::twz_rt_thread_exit(code));
        unreachable!()
    }
}

/// Abort execution due to unrecoverable language error.
pub fn twz_rt_abort() -> ! {
    unsafe {
        nk!(crate::bindings::twz_rt_abort());
        unreachable!()
    }
}

/// Call this before calling main, after initializing the runtime.
/// If this function returns None, then call main. Otherwise, act
/// as if main returned the provided [ExitCode].
pub fn twz_rt_pre_main_hook() -> Option<ExitCode> {
    unsafe { nk!(crate::bindings::twz_rt_pre_main_hook().into()) }
}

impl From<crate::bindings::option_exit_code> for Option<ExitCode> {
    #[inline]
    fn from(value: crate::bindings::option_exit_code) -> Self {
        if value.is_some == 0 {
            None
        } else {
            Some(value.value)
        }
    }
}

/// Call this after return from main, before running destructors.
pub fn twz_rt_post_main_hook() {
    unsafe {
        nk!(crate::bindings::twz_rt_post_main_hook());
    }
}

/// Called by security context code on compartment entry
pub fn twz_rt_cross_compartment_entry() -> Result<(), TwzError> {
    unsafe {
        if nk!(crate::bindings::twz_rt_cross_compartment_entry()) {
            Ok(())
        } else {
            Err(GenericError::AccessDenied.into())
        }
    }
}

pub use crate::bindings::{
    basic_aux as BasicAux, basic_return as BasicReturn, comp_init_info as CompartmentInitInfo,
    ctor_set as CtorSet, init_info_ptrs as InitInfoPtrs, minimal_init_info as MinimalInitInfo,
    runtime_info as RuntimeInfo, RUNTIME_INIT_COMP, RUNTIME_INIT_MIN, RUNTIME_INIT_MONITOR,
};

// Safety: this holds functions pointers, but these pointers have 'static lifetime.
unsafe impl Send for CtorSet {}

// Safety: this holds functions pointers, but these pointers have 'static lifetime.
unsafe impl Sync for CtorSet {}

unsafe impl Send for RuntimeInfo {}
unsafe impl Sync for RuntimeInfo {}
unsafe impl Send for CompartmentInitInfo {}
unsafe impl Sync for CompartmentInitInfo {}

/// Standard ELF aux-vector keys, as used by every libc (and matching mlibc's
/// `options/elf/include/elf.h`).
pub mod auxv {
    /// Terminates the aux vector.
    pub const AT_NULL: usize = 0;
    /// System page size.
    pub const AT_PAGESZ: usize = 6;
    /// Processor feature bits.
    pub const AT_HWCAP: usize = 16;
    /// Nonzero if the program should treat itself as running with elevated privilege.
    pub const AT_SECURE: usize = 23;

    /// Number of `usize` words [`entries`] returns.
    pub const LEN: usize = 8;

    /// The aux vector to append to a C entry stack, as key/value pairs.
    ///
    /// A libc locates the aux vector by walking past the argv and envp terminators of the entry
    /// stack, then reading key/value pairs until it sees [`AT_NULL`]. There is no length anywhere,
    /// so the terminator is what bounds the walk: an entry stack built without one makes
    /// `getauxval()` read off the end of whatever allocation holds the stack.
    ///
    /// Only values that can be answered accurately are included. In particular there is no
    /// `AT_PHDR`/`AT_PHNUM`/`AT_ENTRY` — program headers are served by `twz_rt_iter_phdr`
    /// instead — and no `AT_RANDOM`, since a libc reading it expects a valid pointer to 16 bytes
    /// of entropy. Absent keys make `getauxval()` return 0, which callers must already handle.
    pub const fn entries(page_size: usize) -> [usize; LEN] {
        [
            AT_PAGESZ, page_size,
            // No feature bits are advertised, so string/math routines take their baseline paths.
            AT_HWCAP, 0, AT_SECURE, 0, AT_NULL, 0,
        ]
    }
}

/// The entry point for the runtime. Not for public use.
pub fn twz_rt_runtime_entry(
    info: *const RuntimeInfo,
    std_entry: unsafe extern "C-unwind" fn(BasicAux) -> BasicReturn,
    main: usize,
) -> ! {
    unsafe {
        nk!(crate::bindings::twz_rt_runtime_entry(
            info,
            Some(std_entry),
            main
        ));
        unreachable!()
    }
}

#[cfg(all(feature = "rt0"))]
pub mod rt0 {
    //! rt0 defines a collection of functions that the basic Rust ABI expects to be defined by some
    //! part of the C runtime:
    //!
    //!   - __tls_get_addr for handling non-local TLS regions.

    use super::{BasicAux, BasicReturn, RuntimeInfo};

    // The C-based entry point coming from arch-specific assembly _start function.
    #[no_mangle]
    pub unsafe extern "C" fn __rust_entry_from_c(arg: usize, main: usize) -> ! {
        // Just trampoline to rust-abi code.
        rust_entry(arg as *const _, main)
    }

    /// Entry point for Rust code wishing to start from rt0.
    ///
    /// # Safety
    /// Do not call this unless you are bootstrapping a runtime.
    pub unsafe fn rust_entry(arg: *const RuntimeInfo, main: usize) -> ! {
        // All we need to do is grab the runtime and call its init function. We want to
        // do as little as possible here.
        #[cfg(target_os = "twizzler")]
        {
            super::twz_rt_runtime_entry(arg, std_entry_from_runtime, main)
        }
        #[cfg(not(target_os = "twizzler"))]
        {
            panic!("")
        }
    }

    #[cfg(target_os = "twizzler")]
    extern "C-unwind" {
        fn std_entry_from_runtime(aux: BasicAux) -> BasicReturn;
    }
}
