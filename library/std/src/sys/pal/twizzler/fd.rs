#![unstable(reason = "not public", issue = "none", feature = "fd")]

use crate::io::{self, Read, SeekFrom, ErrorKind};
use crate::io::SeekFrom::{Start, Current, End};

use crate::sys::unsupported;
use crate::os::fd::{FromRawFd, OwnedFd, RawFd, AsRawFd, IntoRawFd, AsFd, BorrowedFd};
use crate::sys_common::{AsInner, FromInner, IntoInner};

use twizzler_runtime_api::SeekFrom as InnerSeek;
use twizzler_runtime_api::{FsError, WriteError, ReadError};

impl core::convert::From<FsError> for io::Error {
    fn from(error: FsError) -> io::Error {
        match error {
            FsError::Other => io::Error::new(io::ErrorKind::Other, "unknown error"),
            FsError::InvalidPath => io::Error::from(ErrorKind::NotFound),
            FsError::LookupError => io::Error::new(ErrorKind::Other, "File Descriptor not found"),
            FsError::SeekError => io::Error::from(ErrorKind::UnexpectedEof),
        }
    }
}

impl core::convert::From<WriteError> for io::Error {
    fn from(error: WriteError) -> io::Error {
        match error {
            WriteError::Other => io::Error::new(io::ErrorKind::Other, "unknown error"),
            WriteError::IoError => io::Error::new(io::ErrorKind::Other, "I/O error"),
            WriteError::PermissionDenied => io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"),
            WriteError::NoIo => io::Error::new(io::ErrorKind::Other, "no I/O on this file"),
        }
    }
}

impl core::convert::From<ReadError> for io::Error {
    fn from(error: ReadError) -> io::Error {
        match error {
            ReadError::Other => io::Error::new(io::ErrorKind::Other, "unknown error"),
            ReadError::IoError => io::Error::new(io::ErrorKind::Other, "I/O error"),
            ReadError::PermissionDenied => io::Error::new(io::ErrorKind::PermissionDenied, "permission denied"),
            ReadError::NoIo => io::Error::new(io::ErrorKind::Other, "no I/O on this file"),
        }
    }
}

// A abstraction that can do continious IO on a set of Twizzler objects
#[derive(Debug)]
pub struct FileDesc {
    pub fd: OwnedFd
}

impl FileDesc {
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let runtime = twizzler_runtime_api::get_runtime();
        let result = runtime.read(self.fd.as_raw_fd(), buf)?;
        Ok(result as usize)
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let runtime = twizzler_runtime_api::get_runtime();
        let result = runtime.write(self.fd.as_raw_fd(), buf)?;
        Ok(result as usize)
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        let runtime = twizzler_runtime_api::get_runtime();
        let inner: InnerSeek = match pos {
            Start(x) => InnerSeek::Start(x),
            End(x) => InnerSeek::End(x),
            Current(x) => InnerSeek::Current(x),
        };

        let result = runtime.seek(self.fd.as_raw_fd(), inner)?;
        Ok(result as u64)
    }

    pub fn duplicate(&self) -> io::Result<FileDesc> {
        self.duplicate_path(&[])
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
