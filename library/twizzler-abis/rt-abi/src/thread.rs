//! Runtime interface for threads.

#![allow(unused_variables)]
use core::{ffi::c_void, time::Duration};

use crate::{error::RawTwzError, nk, Result};

/// Runtime-internal thread ID.
pub type ThreadId = crate::bindings::thread_id;
/// Index of a TLS variable.
pub type TlsIndex = crate::bindings::tls_index;
/// TLS desc
pub type TlsDesc = crate::bindings::tls_desc;
/// Type of a linux-like wait point.
pub type FutexWord = crate::bindings::futex_word;
/// Atomic futex word, for a linux-like thread wait.
pub type AtomicFutexWord = core::sync::atomic::AtomicU32;
/// Arguments to spawn.
pub type ThreadSpawnArgs = crate::bindings::spawn_args;

impl From<Result<(ThreadId, *mut c_void)>> for crate::bindings::spawn_result {
    fn from(value: Result<(ThreadId, *mut c_void)>) -> Self {
        match value {
            Ok((id, tcb)) => Self {
                id,
                tcb,
                err: RawTwzError::success().raw(),
            },
            Err(e) => Self {
                id: 0,
                tcb: core::ptr::null_mut(),
                err: e.raw(),
            },
        }
    }
}

impl From<crate::bindings::spawn_result> for Result<ThreadId> {
    fn from(value: crate::bindings::spawn_result) -> Self {
        let raw = RawTwzError::new(value.err);
        if raw.is_success() {
            Ok(value.id)
        } else {
            Err(raw.error())
        }
    }
}

/// If the futex word pointed to by `word` is equal to expected, put the thread to sleep. This
/// operation is atomic -- the thread is enqueued on the sleep queue _first_, before the equality
/// check. Returns false on timeout, true on all other cases.
pub fn twz_rt_futex_wait(
    word: &AtomicFutexWord,
    expected: FutexWord,
    timeout: Option<Duration>,
) -> Result<()> {
    unsafe {
        match nk!(crate::bindings::twz_rt_futex_wait(
            word.as_ptr().cast(),
            expected,
            timeout.into()
        )) {
            0 => Ok(()),
            e => Err(RawTwzError::new(e).error()),
        }
    }
}

/// Wake up up to max threads waiting on `word`. If max is None, wake up all threads.
pub fn twz_rt_futex_wake(word: &AtomicFutexWord, max: Option<usize>) -> Result<()> {
    let max = raw_max(max);
    unsafe {
        match nk!(crate::bindings::twz_rt_futex_wake(
            word.as_ptr().cast(),
            max
        )) {
            0 => Ok(()),
            e => Err(RawTwzError::new(e).error()),
        }
    }
}

/// As [twz_rt_futex_wake], but reports how many threads were actually woken.
///
/// Callers read zero as "nobody was waiting". libstd's `RwLock` needs that answer: without it, a
/// write-unlock cannot tell whether it notified a writer and wakes every waiting reader as well.
pub fn twz_rt_futex_wake_count(word: &AtomicFutexWord, max: Option<usize>) -> Result<usize> {
    let max = raw_max(max);
    unsafe {
        let res = nk!(crate::bindings::twz_rt_futex_wake_count(
            word.as_ptr().cast(),
            max
        ));
        let raw = RawTwzError::new(res.err);
        if raw.is_success() {
            Ok(res.val as usize)
        } else {
            Err(raw.error())
        }
    }
}

fn raw_max(max: Option<usize>) -> i64 {
    match max {
        Some(max) => max as i64,
        None => crate::bindings::FUTEX_WAKE_ALL,
    }
}

/// Yield the calling thread.
pub fn twz_rt_yield() {
    unsafe {
        nk!(crate::bindings::twz_rt_yield_now());
    }
}

/// Sleep the calling thread for duration `dur`.
pub fn twz_rt_sleep(dur: Duration) {
    unsafe {
        nk!(crate::bindings::twz_rt_sleep(dur.into()));
    }
}

/// Atomic form of the interrupt-generation word.
pub type AtomicInterruptGen = core::sync::atomic::AtomicU64;

/// Get the calling thread's interrupt-generation word.
///
/// Blocking operations sample [twz_rt_interrupt_gen] on entry and report
/// [crate::error::GenericError::Interrupted] once it moves. The raw pointer is exposed because the
/// word doubles as a thread-sync sleep operand, which closes the window between that check and the
/// sleep itself. It is valid only on the calling thread.
pub fn twz_rt_interrupt_word() -> *const AtomicInterruptGen {
    unsafe { nk!(crate::bindings::twz_rt_interrupt_word()).cast() }
}

/// Read the calling thread's interrupt generation.
pub fn twz_rt_interrupt_gen() -> u64 {
    unsafe { &*twz_rt_interrupt_word() }.load(core::sync::atomic::Ordering::Acquire)
}

/// Record that a signal handler which should interrupt blocking operations has run on this thread.
///
/// Called by libc when it dispatches a caught handler with `SA_RESTART` clear. Nothing else should
/// call it: an ignored signal, or one whose handler restarts, must leave blocking calls alone.
pub fn twz_rt_interrupt_bump() {
    unsafe {
        nk!(crate::bindings::twz_rt_interrupt_bump());
    }
}

/// Set the name of the calling thread.
pub fn twz_rt_set_thread_name(name: &core::ffi::CStr) {
    unsafe {
        nk!(crate::bindings::twz_rt_set_name(name.as_ptr()));
    }
}

/// Get the address of a given TLS variable.
pub fn twz_rt_tls_get_addr(index: &TlsIndex) -> *mut u8 {
    unsafe { nk!(crate::bindings::twz_rt_tls_get_addr(index as *const _ as *mut _).cast()) }
}

/// Spawn a thread. On success, that thread starts executing concurrently with the return of this
/// function.
pub fn twz_rt_spawn_thread(args: ThreadSpawnArgs) -> Result<ThreadId> {
    unsafe { nk!(crate::bindings::twz_rt_spawn_thread(args).into()) }
}

/// Wait for a thread to exit, optionally timing out.
pub fn twz_rt_join_thread(id: ThreadId, timeout: Option<Duration>) -> Result<()> {
    unsafe {
        RawTwzError::new(nk!(crate::bindings::twz_rt_join_thread(id, timeout.into()))).result()
    }
}

/// Information the runtime keeps about a live thread.
pub type ThreadInfo = crate::bindings::thread_info;

/// Thread id meaning "the calling thread".
pub const THREAD_ID_SELF: ThreadId = crate::bindings::TWZ_RT_THREAD_ID_SELF;

/// Look up the runtime's record for a thread.
///
/// `tcb` is that thread's TCB pointer, which is what mlibc uses as its `pthread_t`. An id naming
/// no live thread yields a zeroed record with a null `tcb` rather than an error.
pub fn twz_rt_get_thread_info(id: ThreadId) -> ThreadInfo {
    unsafe { nk!(crate::bindings::twz_rt_get_thread_info(id)) }
}
