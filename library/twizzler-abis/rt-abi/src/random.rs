//! Functions for collecting randomness.
use core::mem::MaybeUninit;

use crate::nk;

bitflags::bitflags! {
    /// Possible flags to get random.
    pub struct GetRandomFlags: crate::bindings::get_random_flags {
        /// Do not block. If the operation would block, return fewer than requested bytes.
        const NON_BLOCKING = crate::bindings::GET_RANDOM_NON_BLOCKING;
    }
}

/// Fill up to buf.len() bytes of randomness into the buffer.
pub fn twz_rt_get_random(buf: &mut [MaybeUninit<u8>], flags: GetRandomFlags) -> usize {
    unsafe {
        nk!(crate::bindings::twz_rt_get_random(
            buf.as_mut_ptr().cast(),
            buf.len(),
            flags.bits()
        ))
    }
}
