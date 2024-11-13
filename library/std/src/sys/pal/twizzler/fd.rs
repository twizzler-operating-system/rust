#![unstable(reason = "not public", issue = "none", feature = "fd")]

use crate::io::{self, Read, SeekFrom, IoSlice, IoSliceMut, BorrowedCursor};
use crate::io::SeekFrom::{Start, Current, End};

use crate::sys::unsupported;
use crate::os::fd::{FromRawFd, OwnedFd, RawFd, AsRawFd, IntoRawFd, AsFd, BorrowedFd};
use crate::sys_common::{AsInner, FromInner, IntoInner};

use twizzler_rt_abi::io::{IoError, IoFlags};
use twizzler_rt_abi::io::SeekFrom as InnerSeek;

impl core::convert::From<IoError> for io::Error {
    fn from(_error: IoError) -> io::Error {
        todo!()
    }
}

// A abstraction that can do continious IO on a set of Twizzler objects
#[derive(Debug)]
pub struct FileDesc {
    pub fd: OwnedFd
}

impl FileDesc {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let result = twizzler_rt_abi::io::twz_rt_fd_pread(self.fd.as_raw_fd(), None, buf, IoFlags::empty())?;
        Ok(result as usize)
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let result = twizzler_rt_abi::io::twz_rt_fd_pwrite(self.fd.as_raw_fd(), None, buf, IoFlags::empty())?;
        Ok(result as usize)
    }

    pub fn read_buf(&mut self, mut buf: BorrowedCursor<'_>) -> io::Result<()> {
        let slice = unsafe { core::slice::from_raw_parts_mut(buf.as_mut().as_mut_ptr().cast(), buf.capacity()) };
        let ret = twizzler_rt_abi::io::twz_rt_fd_pread(self.as_raw_fd(), None, slice, IoFlags::empty())?;
        // Safety: `ret` bytes were written to the initialized portion of the buffer
        unsafe {
            buf.advance_unchecked(ret as usize);
        }
        Ok(())
    }

    pub fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let slice = unsafe { core::slice::from_raw_parts(bufs.as_ptr().cast(), bufs.len()) };
        twizzler_rt_abi::io::twz_rt_fd_pwritev(self.as_raw_fd(), None, slice, IoFlags::empty()).map_err(|e| e.into())
    }

    pub fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let slice = unsafe { core::slice::from_raw_parts_mut(bufs.as_mut_ptr().cast(), bufs.len()) };
        twizzler_rt_abi::io::twz_rt_fd_preadv(self.as_raw_fd(), None, slice, IoFlags::empty()).map_err(|e| e.into())
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

    pub fn duplicate(&self) -> io::Result<FileDesc> {
        Ok(unsafe { FileDesc::from_raw_fd(twizzler_rt_abi::fd::twz_rt_fd_dup(self.fd.as_raw_fd())?) })
    }

    pub fn duplicate_path(&self, _path: &[u8]) -> io::Result<FileDesc> {
        unsupported()
    }

    pub fn nonblocking(&self) -> io::Result<bool> {
        Ok(false)
    }

    pub fn set_cloexec(&self) -> io::Result<()> {
        unsupported()
    }

    pub fn set_nonblocking(&self, _nonblocking: bool) -> io::Result<()> {
        unsupported()
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
        Self { fd: FromRawFd::from_raw_fd(raw_fd) }
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
