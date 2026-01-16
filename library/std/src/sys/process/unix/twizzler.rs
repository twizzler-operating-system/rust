#![allow(unused)]

use libc::c_int;

use super::common::*;
use crate::num::NonZero;
use crate::os::fd::{AsRawFd, FromRawFd};
use crate::process::StdioPipes;
use crate::sys::fd::FileDesc;
use crate::{fmt, io, ptr};

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

impl Command {
    pub fn spawn(
        &mut self,
        default: Stdio,
        needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        let envp = self.capture_env();

        if self.saw_nul() {
            return Err(io::const_error!(
                io::ErrorKind::InvalidInput,
                "nul byte found in provided data",
            ));
        }

        let (ours, theirs) = self.setup_io(default, needs_stdin)?;

        let process_handle = unsafe { self.do_exec(theirs, envp.as_ref())? };

        Ok((Process { handle: process_handle }, ours))
    }

    pub fn exec(&mut self, default: Stdio) -> io::Error {
        if self.saw_nul() {
            return io::const_error!(
                io::ErrorKind::InvalidInput,
                "nul byte found in provided data",
            );
        }

        match self.setup_io(default, true) {
            Ok((_, _)) => {
                // FIXME: This is tough because we don't support the exec syscalls
                unimplemented!();
            }
            Err(e) => e,
        }
    }

    fn build_bindings(
        &self,
        stdio: &ChildPipes,
    ) -> io::Result<Vec<twizzler_rt_abi::bindings::binding_info>> {
        let mut bindings = vec![twizzler_rt_abi::bindings::binding_info::default(); 8];

        // First, clone the bindings
        let count = loop {
            let count = twizzler_rt_abi::fd::twz_rt_fd_read_binds(&mut bindings);
            if count < bindings.len() {
                break count;
            }
            bindings.extend(&[twizzler_rt_abi::bindings::binding_info::default(); 8]);
        };
        bindings.truncate(count);

        let stdin_idx = bindings.iter().position(|b| b.fd == 0);
        let stdout_idx = bindings.iter().position(|b| b.fd == 1);
        let stderr_idx = bindings.iter().position(|b| b.fd == 2);

        let mut build = |stdio: &ChildStdio, idx: Option<usize>, fd: i32| -> io::Result<()> {
            match stdio {
                ChildStdio::Inherit => {}
                ChildStdio::Explicit(src_fd) => {
                    let src_idx = bindings.iter().position(|b| b.fd == *src_fd);
                    if let Some(src_idx) = src_idx {
                        if let Some(idx) = idx {
                            bindings[idx] = bindings[src_idx];
                            bindings[idx].fd = fd;
                        } else {
                            let mut binding = bindings[src_idx];
                            binding.fd = fd;
                            bindings.push(binding);
                        }
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "could not find binding info for explicit fd",
                        ));
                    }
                }
                ChildStdio::Owned(src_fd) => {
                    let src_fd = src_fd.as_raw_fd();
                    let src_idx = bindings.iter().position(|b| b.fd == src_fd);
                    if let Some(src_idx) = src_idx {
                        if let Some(idx) = idx {
                            bindings[idx] = bindings[src_idx];
                            bindings[idx].fd = fd;
                        } else {
                            let mut binding = bindings[src_idx];
                            binding.fd = fd;
                            bindings.push(binding);
                        }
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "could not find binding info for explicit fd",
                        ));
                    }
                }
            }
            Ok(())
        };

        build(&stdio.stdin, stdin_idx, 0)?;
        build(&stdio.stdout, stdout_idx, 1)?;
        build(&stdio.stderr, stderr_idx, 2)?;

        Ok(bindings)
    }

    unsafe fn do_exec(
        &mut self,
        stdio: ChildPipes,
        maybe_envp: Option<&CStringArray>,
    ) -> io::Result<FileDesc> {
        let envp = match maybe_envp {
            // None means to clone the current environment, which is done in the
            // flags below.
            None => ptr::null(),
            Some(envp) => envp.as_ptr(),
        };

        let bindings = self.build_bindings(&stdio)?;

        for callback in self.get_closures().iter_mut() {
            callback()?;
        }

        let process_handle = twizzler_rt_abi::exec::twz_rt_exec_spawn(
            self.get_program_cstr(),
            self.get_argv().as_ptr(),
            envp,
            &bindings,
            0,
        )?;
        drop(stdio);
        Ok(FileDesc::from_raw_fd(process_handle))
    }
}

////////////////////////////////////////////////////////////////////////////////
// Processes
////////////////////////////////////////////////////////////////////////////////

pub struct Process {
    handle: FileDesc,
}

impl Process {
    pub fn id(&self) -> u32 {
        self.handle.as_raw_fd().try_into().unwrap()
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.send_signal(libc::SIGKILL)
    }

    pub fn send_signal(&self, signal: i32) -> io::Result<()> {
        let raw = self.handle.as_raw_fd();
        twizzler_rt_abi::io::twz_rt_fd_set_config(
            raw,
            twizzler_rt_abi::bindings::IO_REGISTER_SIGNAL,
            &signal,
        )?;

        Ok(())
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
        }
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        let raw = self.handle.as_raw_fd();
        let mut buf = [0; 1];
        self.handle.read(&mut buf)?;
        let status: u64 = twizzler_rt_abi::io::twz_rt_fd_get_config(
            raw,
            twizzler_rt_abi::bindings::IO_REGISTER_STATUS,
        )?;
        let exit_status = (status & 0xffffffff) as i32;
        let terminated = status & twizzler_rt_abi::bindings::STATUS_FLAG_TERMINATED != 0;
        if terminated { Ok(Some(ExitStatus(exit_status))) } else { Ok(None) }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct ExitStatus(i32);

impl ExitStatus {
    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        match NonZero::try_from(self.0) {
            /* was nonzero */ Ok(failure) => Err(ExitStatusError(failure)),
            /* was zero, couldn't convert */ Err(_) => Ok(()),
        }
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.0)
    }

    pub fn signal(&self) -> Option<i32> {
        if self.0 <= 128 {
            return None;
        }
        Some(self.0 - 128)
    }

    pub fn core_dumped(&self) -> bool {
        false
    }

    pub fn stopped_signal(&self) -> Option<i32> {
        None
    }

    pub fn continued(&self) -> bool {
        false
    }

    pub fn into_raw(&self) -> c_int {
        self.0
    }
}

/// Converts a raw `c_int` to a type-safe `ExitStatus` by wrapping it without copying.
impl From<c_int> for ExitStatus {
    fn from(a: c_int) -> ExitStatus {
        ExitStatus(a as i32)
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exit code: {}", self.0)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitStatusError(NonZero<i32>);

impl Into<ExitStatus> for ExitStatusError {
    fn into(self) -> ExitStatus {
        ExitStatus(self.0.into())
    }
}

impl ExitStatusError {
    pub fn code(self) -> Option<NonZero<i32>> {
        // fixme: affected by the same bug as ExitStatus::code()
        ExitStatus(self.0.into()).code().map(|st| st.try_into().unwrap())
    }
}
