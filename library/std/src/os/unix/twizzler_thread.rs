//! Unix-specific extensions to primitives in the [`std::thread`] module.
//!
//! mlibc provides real pthreads here, and the runtime sets up the mlibc TCB for *every* thread it
//! spawns -- not only those created through `pthread_create`. The reference runtime's spawn path
//! takes a TLS region from the compartment's template (`get_next_tls_info` hands back a
//! `*mut Tcb<RuntimeThreadControl>`), which is why `__mlibc_enter_thread` can simply "get the TCB
//! that was already allocated by the runtime".
//!
//! mlibc's `pthread_t` *is* that TCB pointer, and the runtime already exposes a lookup from a
//! thread id to it: `twz_rt_get_thread_info` returns a `thread_info { id, tcb, objid }`, filled in
//! from the thread manager's per-thread record for any live thread. So `as_pthread_t` here returns
//! a genuine `pthread_t` and may be handed to libc, which matters because real callers do exactly
//! that -- `jobserver` sends `SIGUSR1` to its helper thread with
//! `libc::pthread_kill(handle.as_pthread_t(), ..)`, and `hwlocality` uses it as hwloc's thread id
//! for CPU binding.
//!
//! A thread that has exited, or an id the manager does not know, yields a null `tcb` and therefore
//! `0` -- an invalid `pthread_t`, which is the same answer POSIX gives for a stale handle.
//!
//! [`std::thread`]: crate::thread

#![stable(feature = "thread_extensions", since = "1.9.0")]

use crate::sys::AsInner;
use crate::thread::JoinHandle;

/// mlibc's `pthread_t`: a pointer to the thread's TCB.
#[stable(feature = "thread_extensions", since = "1.9.0")]
pub type RawPthread = libc::pthread_t;

/// Returns the mlibc `pthread_t` for a runtime thread id, or 0 if the thread is not live.
fn pthread_for(id: twizzler_rt_abi::thread::ThreadId) -> RawPthread {
    // Through rt-abi's safe wrapper rather than the raw binding: std denies `ffi_unwind_calls`,
    // and every rt-abi binding is `extern "C-unwind"`.
    twizzler_rt_abi::thread::twz_rt_get_thread_info(id).tcb as RawPthread
}

/// Unix-specific extensions to [`JoinHandle`].
#[stable(feature = "thread_extensions", since = "1.9.0")]
pub trait JoinHandleExt {
    /// Extracts the raw `pthread_t` without taking ownership.
    #[stable(feature = "thread_extensions", since = "1.9.0")]
    fn as_pthread_t(&self) -> RawPthread;

    /// Consumes the handle, returning the raw `pthread_t`.
    ///
    /// Note that unlike a conventional unix, this transfers no ownership: the Twizzler runtime
    /// owns the thread and its TCB, and dropping the handle neither detaches nor joins it.
    #[stable(feature = "thread_extensions", since = "1.9.0")]
    fn into_pthread_t(self) -> RawPthread;
}

#[stable(feature = "thread_extensions", since = "1.9.0")]
impl<T> JoinHandleExt for JoinHandle<T> {
    fn as_pthread_t(&self) -> RawPthread {
        pthread_for(self.as_inner().id())
    }

    fn into_pthread_t(self) -> RawPthread {
        pthread_for(self.as_inner().id())
    }
}
