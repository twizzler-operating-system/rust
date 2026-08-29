//! Functions for interacting with the runtime's support for time, monotonic and system.

use core::time::Duration;

use crate::nk;

/// Possible monotonicities supported by the system monotonic clock.
#[repr(u32)]
pub enum Monotonicity {
    NonMonotonic = crate::bindings::monotonicity_NonMonotonic,
    Weak = crate::bindings::monotonicity_WeakMonotonic,
    Strict = crate::bindings::monotonicity_StrongMonotonic,
}

impl Into<crate::bindings::monotonicity> for Monotonicity {
    fn into(self) -> crate::bindings::monotonicity {
        self as crate::bindings::monotonicity
    }
}

impl From<crate::bindings::monotonicity> for Monotonicity {
    fn from(value: crate::bindings::monotonicity) -> Self {
        match value {
            crate::bindings::monotonicity_WeakMonotonic => Self::Weak,
            crate::bindings::monotonicity_StrongMonotonic => Self::Strict,
            _ => Self::NonMonotonic,
        }
    }
}

/// Read the system monotonic clock.
pub fn twz_rt_get_monotonic_time() -> Duration {
    unsafe { nk!(crate::bindings::twz_rt_get_monotonic_time().into()) }
}

/// Read the system time.
pub fn twz_rt_get_system_time() -> Duration {
    unsafe { nk!(crate::bindings::twz_rt_get_system_time().into()) }
}

impl From<Duration> for crate::bindings::duration {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}

impl From<crate::bindings::duration> for Duration {
    fn from(value: crate::bindings::duration) -> Self {
        Self::new(value.seconds, value.nanos)
    }
}

impl From<Option<Duration>> for crate::bindings::option_duration {
    fn from(value: Option<Duration>) -> Self {
        match value {
            Some(dur) => Self {
                dur: dur.into(),
                is_some: 1,
            },
            None => Self {
                dur: crate::bindings::duration {
                    seconds: 0,
                    nanos: 0,
                },
                is_some: 0,
            },
        }
    }
}
