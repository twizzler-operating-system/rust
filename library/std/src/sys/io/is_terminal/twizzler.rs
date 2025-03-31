use crate::os::fd::{AsFd, AsRawFd};

pub fn is_terminal(fd: &impl AsFd) -> bool {
    let info = twizzler_rt_abi::fd::twz_rt_fd_get_info(fd.as_fd().as_raw_fd());
    info.map(|info| info.flags.contains(twizzler_rt_abi::fd::FdFlags::IS_TERMINAL)).unwrap_or(false)
}
