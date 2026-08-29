//! Functions for getting system information.

use crate::nk;

/// System information.
pub type SystemInfo = crate::bindings::system_info;

impl SystemInfo {
    /// Get the monotonicity of the system monotonic clock.
    pub fn clock_monotonicity(&self) -> crate::time::Monotonicity {
        self.clock_monotonicity.into()
    }
}

/// Get information about the system.
pub fn twz_rt_get_sysinfo() -> SystemInfo {
    unsafe { nk!(crate::bindings::twz_rt_get_sysinfo()) }
}
