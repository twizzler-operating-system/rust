//! A futex condvar that does not signal into an empty room.
//!
//! Identical to `futex.rs` except that it tracks how many threads are inside `wait`, so that
//! `notify_one`/`notify_all` can skip the wake entirely when nobody is waiting. On Twizzler a wake
//! is `sys_thread_sync`, a syscall at roughly a 1.5 us floor, and `notify_*` issues one on every
//! signal whether or not anyone is parked -- which is most of them for the produce-into-a-queue
//! pattern that signals on every item.
//!
//! # Why the counter lives here and not in the runtime
//!
//! The obvious place for this is the runtime's `futex_wake`, as a side table keyed on the futex
//! address. That is unsound on this OS. The kernel resolves `ThreadSyncReference::Virtual32` to an
//! (object, offset) pair (`kernel/src/syscall/sync.rs`), so two compartments that map the same
//! object at different addresses rendezvous on the same sleep queue -- and a per-compartment table
//! would let a waker in one compartment conclude "no waiters" while a waiter sat in another. A lost
//! wakeup is a hang. Keeping the count inside the `Condvar` makes it shared exactly when, and
//! wherever, the futex word itself is shared.
//!
//! # Ordering
//!
//! The dangerous interleaving is a notifier reading `waiters == 0` while a waiter is on its way to
//! sleep. Writing the waiter's steps as A (read the counter), B (register as a waiter) and the
//! notifier's as N1 (bump the counter), N2 (read the waiter count), all four of B, N1 and N2 are
//! `SeqCst` and so share a total order. If N2 reads zero it is ordered before B, hence N1 is too --
//! so by the time the waiter reaches `futex_wait` the bump at N1 is visible, the kernel's own
//! compare-and-sleep sees a value that no longer matches what A read, and the waiter does not
//! sleep. The remaining case, A landing after N1, is the pre-existing contract: a notification
//! delivered without holding the mutex may be missed, exactly as upstream.
//!
//! On x86_64 the strengthening is free -- a `SeqCst` RMW is the same `lock xadd` as a relaxed one,
//! and a `SeqCst` load is a plain `mov`.

use crate::sync::atomic::Ordering::{Relaxed, SeqCst};
use crate::sys::futex::{Futex, futex_wait, futex_wake, futex_wake_all};
use crate::sys::sync::Mutex;
use crate::time::Duration;

pub struct Condvar {
    // The value of this atomic is simply incremented on every notification.
    // This is used by `.wait()` to not miss any notifications after
    // unlocking the mutex and before waiting for notifications.
    futex: Futex,
    // Threads currently inside `wait`. Only ever read to decide whether a wake is worth a syscall;
    // a stale nonzero reading costs one needless syscall, which is the safe direction.
    waiters: Futex,
}

impl Condvar {
    #[inline]
    pub const fn new() -> Self {
        Self { futex: Futex::new(0), waiters: Futex::new(0) }
    }

    /// Whether a wake could reach anybody. See the ordering note above for why this is `SeqCst`.
    #[inline]
    fn has_waiters(&self) -> bool {
        self.waiters.load(SeqCst) != 0
    }

    pub fn notify_one(&self) {
        self.futex.fetch_add(1, SeqCst);
        if self.has_waiters() {
            futex_wake(&self.futex);
        }
    }

    pub fn notify_all(&self) {
        self.futex.fetch_add(1, SeqCst);
        if self.has_waiters() {
            futex_wake_all(&self.futex);
        }
    }

    pub unsafe fn wait(&self, mutex: &Mutex) {
        self.wait_optional_timeout(mutex, None);
    }

    pub unsafe fn wait_timeout(&self, mutex: &Mutex, timeout: Duration) -> bool {
        self.wait_optional_timeout(mutex, Some(timeout))
    }

    unsafe fn wait_optional_timeout(&self, mutex: &Mutex, timeout: Option<Duration>) -> bool {
        // Examine the notification counter _before_ we unlock the mutex.
        let futex_value = self.futex.load(Relaxed);

        // Register before unlocking, so that a notifier holding the mutex cannot get between the
        // two and see an empty room.
        self.waiters.fetch_add(1, SeqCst);

        // Unlock the mutex before going to sleep.
        mutex.unlock();

        // Wait, but only if there hasn't been any
        // notification since we unlocked the mutex.
        let r = futex_wait(&self.futex, futex_value, timeout);

        // Deregister before reacquiring: holding the mutex is not required to be a waiter, and a
        // count left high only costs a future notifier a wasted syscall.
        self.waiters.fetch_sub(1, SeqCst);

        // Lock the mutex again.
        mutex.lock();

        r
    }
}
