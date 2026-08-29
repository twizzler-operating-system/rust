use core::ffi::{c_char, CStr};

use crate::{
    bindings::{binding_info, descriptor, exec_flags, exec_spawn_args},
    error::TwzError,
    nk,
};

pub fn twz_rt_exec_spawn(
    prog: &CStr,
    args: *const *const c_char,
    env: *const *const c_char,
    fd_binds: &[binding_info],
    flags: exec_flags,
) -> Result<descriptor, TwzError> {
    let spawn_args = exec_spawn_args {
        prog: prog.as_ptr(),
        args,
        env,
        fd_binds: fd_binds.as_ptr(),
        fd_bind_count: fd_binds.len(),
        flags,
    };
    unsafe { nk!(crate::bindings::twz_rt_exec_spawn(&spawn_args).into()) }
}
