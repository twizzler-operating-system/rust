#![unstable(reason = "not public", issue = "none", feature = "fd")]

use twizzler_rt_abi::io::SeekFrom as InnerSeek;

use crate::io::SeekFrom::{Current, End, Start};
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, Read, SeekFrom};
use crate::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use crate::sys::{AsInner, FromInner, IntoInner};

// A abstraction that can do continious IO on a set of Twizzler objects
#[derive(Debug)]
pub struct FileDesc {
    pub fd: OwnedFd,
}

impl FileDesc {
    pub fn diverge(&self) -> ! {
        panic!("can't call me")
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut ctx = twizzler_rt_abi::io::IoCtx::default();
        let result = twizzler_rt_abi::io::twz_rt_fd_pread(self.fd.as_raw_fd(), buf, &mut ctx)?;
        Ok(result as usize)
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let mut ctx = twizzler_rt_abi::io::IoCtx::default();
        let result = twizzler_rt_abi::io::twz_rt_fd_pwrite(self.fd.as_raw_fd(), buf, &mut ctx)?;
        Ok(result as usize)
    }

    pub fn read_buf(&self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        crate::io::default_read_buf(|buf| self.read(buf), buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut ctx = twizzler_rt_abi::io::IoCtx::default();
        let slice = unsafe { core::slice::from_raw_parts(bufs.as_ptr().cast(), bufs.len()) };
        twizzler_rt_abi::io::twz_rt_fd_pwritev(self.as_raw_fd(), slice, &mut ctx)
            .map_err(|e| e.into())
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let mut ctx = twizzler_rt_abi::io::IoCtx::default();
        let slice =
            unsafe { core::slice::from_raw_parts_mut(bufs.as_mut_ptr().cast(), bufs.len()) };
        twizzler_rt_abi::io::twz_rt_fd_preadv(self.as_raw_fd(), slice, &mut ctx)
            .map_err(|e| e.into())
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let inner: InnerSeek = match pos {
            Start(x) => InnerSeek::Start(x),
            End(x) => InnerSeek::End(x),
            Current(x) => InnerSeek::Current(x),
        };

        let result = twizzler_rt_abi::io::twz_rt_fd_seek(self.fd.as_raw_fd(), inner)?;
        Ok(result as u64)
    }

    pub fn tell(&self) -> io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }

    pub fn duplicate(&self) -> io::Result<FileDesc> {
        Ok(unsafe {
            FileDesc::from_raw_fd(twizzler_rt_abi::fd::twz_rt_fd_dup(self.fd.as_raw_fd())?)
        })
    }

    fn change_flag(&self, flag: u32, set: bool) -> io::Result<()> {
        let reg = twizzler_rt_abi::io::twz_rt_fd_get_config::<u32>(
            self.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_IO_FLAGS,
        )?;
        let val = if set { reg | flag } else { reg & !flag };
        twizzler_rt_abi::io::twz_rt_fd_set_config::<u32>(
            self.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_IO_FLAGS,
            val,
        )?;
        Ok(())
    }

    fn read_flag(&self, flag: u32) -> io::Result<bool> {
        let reg = twizzler_rt_abi::io::twz_rt_fd_get_config::<u32>(
            self.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_IO_FLAGS,
        )?;
        Ok((reg & flag) != 0)
    }

    pub fn nonblocking(&self) -> io::Result<bool> {
        let val = self.read_flag(twizzler_rt_abi::bindings::IO_NONBLOCKING)?;
        Ok(val)
    }

    pub fn set_cloexec(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.change_flag(twizzler_rt_abi::bindings::IO_NONBLOCKING, nonblocking)?;
        Ok(())
    }

    pub fn is_write_vectored(&self) -> bool {
        // TODO: use twizzler vec io
        false
    }

    pub fn is_read_vectored(&self) -> bool {
        // TODO: use twizzler vec io
        false
    }

    pub fn try_clone(&self) -> io::Result<Self> {
        self.duplicate()
    }
}

impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }
}

impl IntoInner<OwnedFd> for FileDesc {
    fn into_inner(self) -> OwnedFd {
        self.fd
    }
}

impl FromInner<OwnedFd> for FileDesc {
    fn from_inner(owned_fd: OwnedFd) -> Self {
        Self { fd: owned_fd }
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self { fd: unsafe { FromRawFd::from_raw_fd(raw_fd) } }
    }
}

impl AsInner<OwnedFd> for FileDesc {
    #[inline]
    fn as_inner(&self) -> &OwnedFd {
        &self.fd
    }
}

impl AsFd for FileDesc {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl AsRawFd for FileDesc {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        self.fd.into_raw_fd()
    }
}
