//! `std::os::unix::net` for Twizzler.
//!
//! There is no AF_UNIX transport here, and this file does not fake one: every operation that
//! would create or address a socket fails with [`io::ErrorKind::Unsupported`]. What is real is
//! `UnixStream::pair` (the runtime backs `socketpair(AF_UNIX, SOCK_STREAM)` with a cross-wired
//! pipe pair) and the descriptor plumbing -- the fd traits, `try_clone`, `set_nonblocking`,
//! `shutdown`, and `Read`/`Write` on `UnixStream` -- so a descriptor that arrives through
//! `FromRawFd` behaves like the fd it is. The point is that code written against the portable
//! API compiles and fails at the call that actually needs a named unix socket, not at link or
//! module resolution.

#![stable(feature = "unix_socket", since = "1.10.0")]

use crate::fmt;
use crate::io::{self, IoSlice, IoSliceMut};
use crate::net::Shutdown;
use crate::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use crate::path::{Path, PathBuf};
use crate::sys::fd::FileDesc;
use crate::sys::{cvt, unsupported};
use crate::sys::{FromInner, IntoInner};
use crate::time::Duration;

/// An address associated with a Unix socket. Only pathname and unnamed forms exist.
#[derive(Clone)]
#[stable(feature = "unix_socket", since = "1.10.0")]
pub struct SocketAddr {
    path: Option<PathBuf>,
}

impl SocketAddr {
    /// Constructs a `SocketAddr` with the family `AF_UNIX` and the provided path.
    #[stable(feature = "unix_socket_creation", since = "1.61.0")]
    pub fn from_pathname<P>(path: P) -> io::Result<SocketAddr>
    where
        P: AsRef<Path>,
    {
        Ok(SocketAddr { path: Some(path.as_ref().to_path_buf()) })
    }

    /// Returns `true` if the address is unnamed.
    #[must_use]
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn is_unnamed(&self) -> bool {
        self.path.is_none()
    }

    /// Returns the contents of this address if it is a `pathname` address.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    #[must_use]
    pub fn as_pathname(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

#[stable(feature = "unix_socket", since = "1.10.0")]
impl fmt::Debug for SocketAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            None => write!(fmt, "(unnamed)"),
            Some(path) => write!(fmt, "{path:?} (pathname)"),
        }
    }
}

macro_rules! fd_traits {
    ($t:ident) => {
        #[stable(feature = "unix_socket", since = "1.10.0")]
        impl AsRawFd for $t {
            #[inline]
            fn as_raw_fd(&self) -> RawFd {
                self.0.as_raw_fd()
            }
        }

        #[stable(feature = "unix_socket", since = "1.10.0")]
        impl FromRawFd for $t {
            #[inline]
            unsafe fn from_raw_fd(fd: RawFd) -> $t {
                $t(unsafe { FileDesc::from_raw_fd(fd) })
            }
        }

        #[stable(feature = "unix_socket", since = "1.10.0")]
        impl IntoRawFd for $t {
            #[inline]
            fn into_raw_fd(self) -> RawFd {
                self.0.into_raw_fd()
            }
        }

        #[stable(feature = "io_safety", since = "1.63.0")]
        impl AsFd for $t {
            #[inline]
            fn as_fd(&self) -> BorrowedFd<'_> {
                self.0.as_fd()
            }
        }

        #[stable(feature = "io_safety", since = "1.63.0")]
        impl From<$t> for OwnedFd {
            /// Takes ownership of the underlying file descriptor.
            #[inline]
            fn from(v: $t) -> OwnedFd {
                v.0.into_inner()
            }
        }

        #[stable(feature = "io_safety", since = "1.63.0")]
        impl From<OwnedFd> for $t {
            #[inline]
            fn from(owned: OwnedFd) -> Self {
                $t(FileDesc::from_inner(owned))
            }
        }

        #[stable(feature = "unix_socket", since = "1.10.0")]
        impl fmt::Debug for $t {
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt.debug_struct(stringify!($t)).field("fd", &self.0.as_raw_fd()).finish()
            }
        }
    };
}

/// A Unix stream socket.
#[stable(feature = "unix_socket", since = "1.10.0")]
pub struct UnixStream(FileDesc);
fd_traits!(UnixStream);

impl UnixStream {
    /// Connects to the socket named by `path`.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn connect<P: AsRef<Path>>(_path: P) -> io::Result<UnixStream> {
        unsupported()
    }

    /// Connects to an address.
    #[stable(feature = "unix_socket_abstract", since = "1.70.0")]
    pub fn connect_addr(_socket_addr: &SocketAddr) -> io::Result<UnixStream> {
        unsupported()
    }

    /// Creates an unnamed pair of connected sockets.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn pair() -> io::Result<(UnixStream, UnixStream)> {
        let mut fds = [0 as libc::c_int; 2];
        cvt(unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) })?;
        Ok((
            UnixStream(unsafe { FileDesc::from_raw_fd(fds[0]) }),
            UnixStream(unsafe { FileDesc::from_raw_fd(fds[1]) }),
        ))
    }

    /// Creates a new independently owned handle to the underlying socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn try_clone(&self) -> io::Result<UnixStream> {
        self.0.try_clone().map(UnixStream)
    }

    /// Returns the socket address of the local half of this connection.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        unsupported()
    }

    /// Returns the socket address of the remote half of this connection.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        unsupported()
    }

    /// Sets the read timeout for the socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn set_read_timeout(&self, _timeout: Option<Duration>) -> io::Result<()> {
        unsupported()
    }

    /// Sets the write timeout for the socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn set_write_timeout(&self, _timeout: Option<Duration>) -> io::Result<()> {
        unsupported()
    }

    /// Returns the read timeout of this socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        unsupported()
    }

    /// Returns the write timeout of this socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        unsupported()
    }

    /// Moves the socket into or out of nonblocking mode.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }

    /// Returns the value of the `SO_ERROR` option.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        Ok(None)
    }

    /// Shuts down the read, write, or both halves of this connection.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let how = match how {
            Shutdown::Read => libc::SHUT_RD,
            Shutdown::Write => libc::SHUT_WR,
            Shutdown::Both => libc::SHUT_RDWR,
        };
        cvt(unsafe { libc::shutdown(self.0.as_raw_fd(), how) }).map(drop)
    }

    /// Receives data on the socket from the remote address to which it is connected, without
    /// removing that data from the queue.
    #[unstable(feature = "unix_socket_peek", issue = "76923")]
    pub fn peek(&self, _buf: &mut [u8]) -> io::Result<usize> {
        unsupported()
    }
}

#[stable(feature = "unix_socket", since = "1.10.0")]
impl io::Read for UnixStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        io::Read::read(&mut &*self, buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        io::Read::read_vectored(&mut &*self, bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        io::Read::is_read_vectored(&&*self)
    }
}

#[stable(feature = "unix_socket", since = "1.10.0")]
impl<'a> io::Read for &'a UnixStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }
}

#[stable(feature = "unix_socket", since = "1.10.0")]
impl io::Write for UnixStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::Write::write(&mut &*self, buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        io::Write::write_vectored(&mut &*self, bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        io::Write::is_write_vectored(&&*self)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::Write::flush(&mut &*self)
    }
}

#[stable(feature = "unix_socket", since = "1.10.0")]
impl<'a> io::Write for &'a UnixStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A structure representing a Unix domain socket server.
#[stable(feature = "unix_socket", since = "1.10.0")]
pub struct UnixListener(FileDesc);
fd_traits!(UnixListener);

impl UnixListener {
    /// Creates a new `UnixListener` bound to the specified socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn bind<P: AsRef<Path>>(_path: P) -> io::Result<UnixListener> {
        unsupported()
    }

    /// Creates a new `UnixListener` bound to the specified [`socket address`].
    #[stable(feature = "unix_socket_abstract", since = "1.70.0")]
    pub fn bind_addr(_socket_addr: &SocketAddr) -> io::Result<UnixListener> {
        unsupported()
    }

    /// Accepts a new incoming connection to this listener.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn accept(&self) -> io::Result<(UnixStream, SocketAddr)> {
        unsupported()
    }

    /// Creates a new independently owned handle to the underlying socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn try_clone(&self) -> io::Result<UnixListener> {
        self.0.try_clone().map(UnixListener)
    }

    /// Returns the local socket address of this listener.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        unsupported()
    }

    /// Moves the socket into or out of nonblocking mode.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }

    /// Returns the value of the `SO_ERROR` option.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        Ok(None)
    }

    /// Returns an iterator over incoming connections.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }
}

#[stable(feature = "unix_socket", since = "1.10.0")]
impl<'a> IntoIterator for &'a UnixListener {
    type Item = io::Result<UnixStream>;
    type IntoIter = Incoming<'a>;

    fn into_iter(self) -> Incoming<'a> {
        self.incoming()
    }
}

/// An iterator over incoming connections to a [`UnixListener`].
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[stable(feature = "unix_socket", since = "1.10.0")]
pub struct Incoming<'a> {
    listener: &'a UnixListener,
}

#[stable(feature = "unix_socket", since = "1.10.0")]
impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<UnixStream>;

    fn next(&mut self) -> Option<io::Result<UnixStream>> {
        Some(self.listener.accept().map(|s| s.0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

/// A Unix datagram socket.
#[stable(feature = "unix_socket", since = "1.10.0")]
pub struct UnixDatagram(FileDesc);
fd_traits!(UnixDatagram);

impl UnixDatagram {
    /// Creates a Unix datagram socket bound to the given path.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn bind<P: AsRef<Path>>(_path: P) -> io::Result<UnixDatagram> {
        unsupported()
    }

    /// Creates a Unix datagram socket bound to an address.
    #[stable(feature = "unix_socket_abstract", since = "1.70.0")]
    pub fn bind_addr(_socket_addr: &SocketAddr) -> io::Result<UnixDatagram> {
        unsupported()
    }

    /// Creates a Unix Datagram socket which is not bound to any address.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn unbound() -> io::Result<UnixDatagram> {
        unsupported()
    }

    /// Creates an unnamed pair of connected sockets.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn pair() -> io::Result<(UnixDatagram, UnixDatagram)> {
        unsupported()
    }

    /// Connects the socket to the specified path address.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn connect<P: AsRef<Path>>(&self, _path: P) -> io::Result<()> {
        unsupported()
    }

    /// Connects the socket to an address.
    #[stable(feature = "unix_socket_abstract", since = "1.70.0")]
    pub fn connect_addr(&self, _socket_addr: &SocketAddr) -> io::Result<()> {
        unsupported()
    }

    /// Creates a new independently owned handle to the underlying socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn try_clone(&self) -> io::Result<UnixDatagram> {
        self.0.try_clone().map(UnixDatagram)
    }

    /// Returns the address of this socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        unsupported()
    }

    /// Returns the address of this socket's peer.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        unsupported()
    }

    /// Receives data from the socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn recv_from(&self, _buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        unsupported()
    }

    /// Receives data from the socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn recv(&self, _buf: &mut [u8]) -> io::Result<usize> {
        unsupported()
    }

    /// Sends data on the socket to the specified address.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn send_to<P: AsRef<Path>>(&self, _buf: &[u8], _path: P) -> io::Result<usize> {
        unsupported()
    }

    /// Sends data on the socket to the specified [SocketAddr].
    #[stable(feature = "unix_socket_abstract", since = "1.70.0")]
    pub fn send_to_addr(&self, _buf: &[u8], _socket_addr: &SocketAddr) -> io::Result<usize> {
        unsupported()
    }

    /// Sends data on the socket to the socket's peer.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn send(&self, _buf: &[u8]) -> io::Result<usize> {
        unsupported()
    }

    /// Sets the read timeout for the socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn set_read_timeout(&self, _timeout: Option<Duration>) -> io::Result<()> {
        unsupported()
    }

    /// Sets the write timeout for the socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn set_write_timeout(&self, _timeout: Option<Duration>) -> io::Result<()> {
        unsupported()
    }

    /// Returns the read timeout of this socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        unsupported()
    }

    /// Returns the write timeout of this socket.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        unsupported()
    }

    /// Moves the socket into or out of nonblocking mode.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }

    /// Returns the value of the `SO_ERROR` option.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        Ok(None)
    }

    /// Shut down the read, write, or both halves of this connection.
    #[stable(feature = "unix_socket", since = "1.10.0")]
    pub fn shutdown(&self, _how: Shutdown) -> io::Result<()> {
        unsupported()
    }

    /// Receives data on the socket from the remote address to which it is connected, without
    /// removing that data from the queue.
    #[unstable(feature = "unix_socket_peek", issue = "76923")]
    pub fn peek(&self, _buf: &mut [u8]) -> io::Result<usize> {
        unsupported()
    }

    /// Receives a single datagram message on the socket, without removing it from the queue.
    #[unstable(feature = "unix_socket_peek", issue = "76923")]
    pub fn peek_from(&self, _buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        unsupported()
    }
}
