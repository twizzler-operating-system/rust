use crate::io;
#[cfg(target_os = "twizzler")]
use crate::os::fd::AsRawFd;
use crate::os::fd::FromRawFd;
use crate::sys::fd::FileDesc;
#[cfg(not(target_os = "twizzler"))]
use crate::sys::pal::cvt;

pub type Pipe = FileDesc;

pub fn pipe() -> io::Result<(Pipe, Pipe)> {
    // The only known way right now to create atomically set the CLOEXEC flag is
    // to use the `pipe2` syscall. This was added to Linux in 2.6.27, glibc 2.9
    // and musl 0.9.3, and some other targets also have it.
    cfg_select! {
        any(
            target_os = "android",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "hurd",
            target_os = "illumos",
            target_os = "linux",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "cygwin",
            target_os = "redox"
        ) => {
            let mut fds = [0; 2];
            unsafe {
                cvt(libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC))?;
                Ok((Pipe::from_raw_fd(fds[0]), Pipe::from_raw_fd(fds[1])))
            }
        }
        target_os = "twizzler" => {
            let fd = twizzler_rt_abi::fd::twz_rt_fd_open_pipe(None, twizzler_rt_abi::bindings::OPEN_FLAG_READ | twizzler_rt_abi::bindings::OPEN_FLAG_WRITE)?;
            let pipe = unsafe {Pipe::from_raw_fd(fd)};
            let pipe2 = pipe.duplicate()?;
            twizzler_rt_abi::fd::twz_rt_fd_shutdown(pipe.as_raw_fd(), false, true)?;
            twizzler_rt_abi::fd::twz_rt_fd_shutdown(pipe2.as_raw_fd(), true, false)?;
            // anon_pipe semantics: cloexec on both ends, like pipe2(O_CLOEXEC) above. Without
            // this the exec path's inherit-all keeps both ends, and a spawned child receives
            // the write end of its own stdin pipe -- so EOF-on-parent-close never arrives.
            twizzler_rt_abi::fd::twz_rt_fd_set_cloexec(pipe.as_raw_fd(), true)?;
            twizzler_rt_abi::fd::twz_rt_fd_set_cloexec(pipe2.as_raw_fd(), true)?;
            Ok((pipe, pipe2))
        }
        _ => {
            let mut fds = [0; 2];
            unsafe {
                cvt(libc::pipe(fds.as_mut_ptr()))?;

                let fd0 = Pipe::from_raw_fd(fds[0]);
                let fd1 = Pipe::from_raw_fd(fds[1]);
                fd0.set_cloexec()?;
                fd1.set_cloexec()?;
                Ok((fd0, fd1))
            }
        }
    }
}
