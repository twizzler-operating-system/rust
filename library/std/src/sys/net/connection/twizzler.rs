#![allow(unused_variables)]
#![allow(dead_code)]

use libc::MSG_PEEK;
use twizzler_rt_abi::fd::ProtKind;
use twizzler_rt_abi::io::IoFlags;

use crate::net::ToSocketAddrs;
use crate::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, RawFd};
use crate::sys::fd::FileDesc;
use crate::sys::{AsInner, FromInner, IntoInner};

#[derive(Debug)]
pub struct Socket(FileDesc, ProtKind);

impl Socket {
    pub fn new(fam: i32, ty: i32) -> io::Result<Socket> {
        let prot = match ty {
            libc::SOCK_STREAM => twizzler_rt_abi::fd::ProtKind::Stream,
            libc::SOCK_DGRAM => twizzler_rt_abi::fd::ProtKind::Datagram,
            _ => return unsupported(),
        };
        let addr: SocketAddr = match fam {
            libc::AF_INET => (Ipv4Addr::UNSPECIFIED, 0).into(),
            libc::AF_INET6 => (Ipv6Addr::UNSPECIFIED, 0).into(),
            _ => return unsupported(),
        };
        let fd = twizzler_rt_abi::fd::twz_rt_fd_open_socket(0, prot)?;

        Ok(Self(unsafe { FileDesc::from_raw_fd(fd) }, prot))
    }

    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        let mut res = Ok(());
        for addr in addr.to_socket_addrs()? {
            let thisres = twizzler_rt_abi::fd::twz_rt_fd_socket_reconnect(
                self.0.as_raw_fd(),
                addr.into(),
                0,
                self.1,
            );
            if thisres.is_ok() {
                return Ok(());
            }
            res = thisres.into();
        }
        Ok(res?)
    }

    pub fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> io::Result<()> {
        let old_timeout = self.write_timeout()?;
        self.set_write_timeout(Some(timeout))?;
        let res = self.connect(addr);
        let _ = self.set_write_timeout(old_timeout);
        res
    }

    pub fn duplicate(&self) -> io::Result<Socket> {
        Ok(Self(self.0.duplicate()?, self.1))
    }

    fn recv_with_flags(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        let mut iof = IoFlags::empty();
        if flags & libc::MSG_WAITALL != 0 {
            iof.insert(IoFlags::WAITALL);
        }
        if flags & libc::MSG_OOB != 0 {
            iof.insert(IoFlags::OOB);
        }
        if flags & libc::MSG_PEEK != 0 {
            iof.insert(IoFlags::PEEK);
        }

        let mut ctx = twizzler_rt_abi::io::IoCtx::default().flags(iof);
        let result = twizzler_rt_abi::io::twz_rt_fd_pread(self.0.as_raw_fd(), buf, &mut ctx)?;
        Ok(result as usize)
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv_with_flags(buf, MSG_PEEK)
    }

    pub fn read_buf(&self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    fn recv_from_with_flags(&self, buf: &mut [u8], flags: i32) -> io::Result<(usize, SocketAddr)> {
        let mut iof = IoFlags::empty();
        if flags & libc::MSG_WAITALL != 0 {
            iof.insert(IoFlags::WAITALL);
        }
        if flags & libc::MSG_OOB != 0 {
            iof.insert(IoFlags::OOB);
        }
        if flags & libc::MSG_PEEK != 0 {
            iof.insert(IoFlags::PEEK);
        }

        let mut ctx = twizzler_rt_abi::io::IoCtx::default().flags(iof);
        let result = twizzler_rt_abi::io::twz_rt_fd_pread_from(self.0.as_raw_fd(), buf, &mut ctx)?;
        let addr: twizzler_rt_abi::fd::SocketAddress = result.1.try_into()?;
        Ok((result.0 as usize, addr.into()))
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, 0)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_with_flags(buf, MSG_PEEK)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    pub fn set_timeout(&self, dur: Option<Duration>, kind: i32) -> io::Result<()> {
        let millis = dur.map_or(0, |d| d.as_millis() as u64);
        twizzler_rt_abi::io::twz_rt_fd_set_config::<u64>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_READTIMEOUT,
            millis,
        )?;
        Ok(())
    }

    pub fn timeout(&self, kind: i32) -> io::Result<Option<Duration>> {
        let millis = twizzler_rt_abi::io::twz_rt_fd_get_config::<u64>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_READTIMEOUT,
        )?;
        if millis == 0 {
            return Ok(None);
        }
        Ok(Some(Duration::from_millis(millis)))
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let (r, w) = match how {
            Shutdown::Read => (true, false),
            Shutdown::Write => (false, true),
            Shutdown::Both => (true, true),
        };
        twizzler_rt_abi::fd::twz_rt_fd_shutdown(self.0.as_raw_fd(), r, w)?;
        Ok(())
    }

    pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
        let millis = linger.map_or(0, |d| d.as_millis() as u64);
        twizzler_rt_abi::io::twz_rt_fd_set_config::<u64>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_LINGER,
            millis,
        )?;
        Ok(())
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        let millis = twizzler_rt_abi::io::twz_rt_fd_get_config::<u64>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_LINGER,
        )?;
        if millis == 0 {
            return Ok(None);
        }
        Ok(Some(Duration::from_millis(millis)))
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        let reg = twizzler_rt_abi::io::twz_rt_fd_get_config::<u32>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_SOCKET_FLAGS,
        )?;
        let reg = twizzler_rt_abi::io::twz_rt_fd_set_config::<u32>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_SOCKET_FLAGS,
            reg | twizzler_rt_abi::bindings::SOCKET_FLAGS_NODELAY,
        )?;
        Ok(())
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        let reg = twizzler_rt_abi::io::twz_rt_fd_get_config::<u32>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_SOCKET_FLAGS,
        )?;
        Ok((reg & twizzler_rt_abi::bindings::SOCKET_FLAGS_NODELAY) != 0)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        Ok(None)
    }

    // This is used by sys_common code to abstract over Windows and Unix.
    pub fn as_raw(&self) -> RawFd {
        self.0.as_raw_fd()
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.set_timeout(dur, libc::SO_RCVTIMEO)
    }

    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.set_timeout(dur, libc::SO_SNDTIMEO)
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.timeout(libc::SO_RCVTIMEO)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.timeout(libc::SO_SNDTIMEO)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        let addr = twizzler_rt_abi::io::twz_rt_fd_get_config::<twizzler_rt_abi::fd::SocketAddress>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_PEER,
        )?;
        Ok(addr.into())
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        let addr = twizzler_rt_abi::io::twz_rt_fd_get_config::<twizzler_rt_abi::fd::SocketAddress>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_ADDR,
        )?;
        Ok(addr.into())
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        twizzler_rt_abi::io::twz_rt_fd_set_config::<u32>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_TTL,
            ttl,
        )?;
        Ok(())
    }

    pub fn ttl(&self) -> io::Result<u32> {
        let val = twizzler_rt_abi::io::twz_rt_fd_get_config::<u32>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_TTL,
        )?;
        Ok(val)
    }

    pub fn set_only_v6(&self, _: bool) -> io::Result<()> {
        let reg = twizzler_rt_abi::io::twz_rt_fd_get_config::<u32>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_SOCKET_FLAGS,
        )?;
        let reg = twizzler_rt_abi::io::twz_rt_fd_set_config::<u32>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_SOCKET_FLAGS,
            reg | twizzler_rt_abi::bindings::SOCKET_FLAGS_ONLYV6,
        )?;
        Ok(())
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        let reg = twizzler_rt_abi::io::twz_rt_fd_get_config::<u32>(
            self.0.as_raw_fd(),
            twizzler_rt_abi::bindings::IO_REGISTER_SOCKET_FLAGS,
        )?;
        Ok((reg & twizzler_rt_abi::bindings::SOCKET_FLAGS_ONLYV6) != 0)
    }

    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        let mut ctx = twizzler_rt_abi::io::IoCtx::default();
        let addr: twizzler_rt_abi::fd::SocketAddress = (*addr).into();
        let result = twizzler_rt_abi::io::twz_rt_fd_pwrite_to(
            self.0.as_raw_fd(),
            buf,
            &mut ctx,
            addr.into(),
        )?;
        Ok(result as usize)
    }
}

impl AsInner<FileDesc> for Socket {
    #[inline]
    fn as_inner(&self) -> &FileDesc {
        &self.0
    }
}

impl IntoInner<FileDesc> for Socket {
    fn into_inner(self) -> FileDesc {
        self.0
    }
}

impl FromInner<FileDesc> for Socket {
    fn from_inner(file_desc: FileDesc) -> Self {
        Self(file_desc, ProtKind::Stream)
    }
}

impl AsFd for Socket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for Socket {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr};
use crate::sys::unsupported;
use crate::time::Duration;

#[derive(Debug)]
pub struct TcpStream(Socket);

impl TcpStream {
    pub fn socket(&self) -> &Socket {
        &self.0
    }

    pub fn into_socket(self) -> Socket {
        self.0
    }

    pub fn connect<A: ToSocketAddrs>(a: A) -> io::Result<TcpStream> {
        let socket = Socket::new(libc::AF_INET, libc::SOCK_STREAM)?;
        socket.connect(a)?;
        Ok(Self(socket))
    }

    pub fn connect_timeout(a: &SocketAddr, d: Duration) -> io::Result<TcpStream> {
        let socket = Socket::new(libc::AF_INET, libc::SOCK_STREAM)?;
        socket.connect_timeout(a, d)?;
        Ok(Self(socket))
    }

    pub fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(t)
    }

    pub fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(t)
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.read_timeout()
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.write_timeout()
    }

    pub fn peek(&self, b: &mut [u8]) -> io::Result<usize> {
        self.0.peek(b)
    }

    pub fn read(&self, b: &mut [u8]) -> io::Result<usize> {
        self.0.read(b)
    }

    pub fn read_buf(&self, buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.0.read_buf(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    pub fn write(&self, b: &[u8]) -> io::Result<usize> {
        self.0.write(b)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    pub fn shutdown(&self, s: Shutdown) -> io::Result<()> {
        self.0.shutdown(s)
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        Ok(Self(self.0.duplicate()?))
    }

    pub fn set_linger(&self, b: Option<Duration>) -> io::Result<()> {
        self.0.set_linger(b)
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        self.0.linger()
    }

    pub fn set_nodelay(&self, b: bool) -> io::Result<()> {
        self.0.set_nodelay(b)
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.0.nodelay()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    pub fn set_nonblocking(&self, b: bool) -> io::Result<()> {
        self.0.set_nonblocking(b)
    }
}

#[derive(Debug)]
pub struct TcpListener(Socket);

impl TcpListener {
    pub fn socket(&self) -> &Socket {
        &self.0
    }

    pub fn into_socket(self) -> Socket {
        self.0
    }

    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        let socket = Socket::new(libc::AF_INET, libc::SOCK_STREAM)?;
        let mut res: io::Result<TcpListener> = Err(crate::io::ErrorKind::HostUnreachable.into());
        for addr in addr.to_socket_addrs()? {
            let thisres = twizzler_rt_abi::fd::twz_rt_fd_socket_rebind(
                socket.0.as_raw_fd(),
                addr.into(),
                0,
                ProtKind::Stream,
            );
            res = match thisres {
                Ok(_) => return Ok(Self(socket)),
                Err(e) => Err(e.into()),
            };
        }
        res
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let res = twizzler_rt_abi::fd::twz_rt_fd_open_socket_accept(self.0.as_raw_fd(), 0)?;
        let socket = Socket(unsafe { FileDesc::from_raw_fd(res) }, ProtKind::Stream);
        let addr = socket.peer_addr()?;
        Ok((TcpStream(socket), addr))
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        Ok(Self(self.0.duplicate()?))
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    pub fn set_only_v6(&self, only_v6: bool) -> io::Result<()> {
        self.0.set_only_v6(only_v6)
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        self.0.only_v6()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    pub fn set_nonblocking(&self, b: bool) -> io::Result<()> {
        self.0.set_nonblocking(b)
    }
}

#[derive(Debug)]
pub struct UdpSocket(Socket);

impl UdpSocket {
    pub fn socket(&self) -> &Socket {
        &self.0
    }

    pub fn into_socket(self) -> Socket {
        self.0
    }

    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket> {
        let socket = Socket::new(libc::AF_INET, libc::SOCK_DGRAM)?;
        let mut res: io::Result<UdpSocket> = Err(crate::io::ErrorKind::HostUnreachable.into());
        for addr in addr.to_socket_addrs()? {
            let thisres = twizzler_rt_abi::fd::twz_rt_fd_socket_rebind(
                socket.0.as_raw_fd(),
                addr.into(),
                0,
                socket.1,
            );
            res = match thisres {
                Ok(_) => return Ok(Self(socket)),
                Err(e) => Err(e.into()),
            };
        }
        Ok(res?)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.recv_from(buf)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.peek_from(buf)
    }

    pub fn send_to(&self, buf: &[u8], addr: &SocketAddr) -> io::Result<usize> {
        self.0.send_to(buf, addr)
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        Ok(Self(self.0.duplicate()?))
    }

    pub fn set_read_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(t)
    }

    pub fn set_write_timeout(&self, t: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(t)
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.read_timeout()
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0.write_timeout()
    }

    pub fn set_broadcast(&self, _: bool) -> io::Result<()> {
        unsupported()
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        unsupported()
    }

    pub fn set_multicast_loop_v4(&self, _: bool) -> io::Result<()> {
        unsupported()
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        unsupported()
    }

    pub fn set_multicast_ttl_v4(&self, _: u32) -> io::Result<()> {
        unsupported()
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        unsupported()
    }

    pub fn set_multicast_loop_v6(&self, _: bool) -> io::Result<()> {
        unsupported()
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        unsupported()
    }

    pub fn join_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr) -> io::Result<()> {
        unsupported()
    }

    pub fn join_multicast_v6(&self, _: &Ipv6Addr, _: u32) -> io::Result<()> {
        unsupported()
    }

    pub fn leave_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr) -> io::Result<()> {
        unsupported()
    }

    pub fn leave_multicast_v6(&self, _: &Ipv6Addr, _: u32) -> io::Result<()> {
        unsupported()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.0.set_ttl(ttl)
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.0.ttl()
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0.take_error()
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.0.set_nonblocking(nonblocking)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.peek(buf)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        let mut res = Ok(());
        for addr in addr.to_socket_addrs()? {
            let addr = twizzler_rt_abi::fd::SocketAddress::from(addr);
            let thisres = twizzler_rt_abi::fd::twz_rt_fd_socket_reconnect(
                self.0.as_raw_fd(),
                addr,
                0,
                self.0.1,
            );
            if thisres.is_ok() {
                return Ok(());
            }
            res = thisres.into();
        }
        Ok(res?)
    }
}

pub struct LookupHost(!);

impl LookupHost {
    pub fn port(&self) -> u16 {
        self.0
    }
}

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        self.0
    }
}

impl TryFrom<&str> for LookupHost {
    type Error = io::Error;

    fn try_from(_v: &str) -> io::Result<LookupHost> {
        unsupported()
    }
}

impl<'a> TryFrom<(&'a str, u16)> for LookupHost {
    type Error = io::Error;

    fn try_from(_v: (&'a str, u16)) -> io::Result<LookupHost> {
        unsupported()
    }
}

impl AsInner<Socket> for TcpStream {
    #[inline]
    fn as_inner(&self) -> &Socket {
        &self.0
    }
}

impl IntoInner<Socket> for TcpStream {
    fn into_inner(self) -> Socket {
        self.0
    }
}

impl FromInner<Socket> for TcpStream {
    fn from_inner(file_desc: Socket) -> Self {
        Self(file_desc)
    }
}

impl AsInner<Socket> for TcpListener {
    #[inline]
    fn as_inner(&self) -> &Socket {
        &self.0
    }
}

impl IntoInner<Socket> for TcpListener {
    fn into_inner(self) -> Socket {
        self.0
    }
}

impl FromInner<Socket> for TcpListener {
    fn from_inner(file_desc: Socket) -> Self {
        Self(file_desc)
    }
}

impl AsInner<Socket> for UdpSocket {
    #[inline]
    fn as_inner(&self) -> &Socket {
        &self.0
    }
}

impl IntoInner<Socket> for UdpSocket {
    fn into_inner(self) -> Socket {
        self.0
    }
}

impl FromInner<Socket> for UdpSocket {
    fn from_inner(file_desc: Socket) -> Self {
        Self(file_desc)
    }
}

pub fn lookup_host(_host: &str, _port: u16) -> io::Result<LookupHost> {
    unsupported()
}
