#![allow(unused_variables)]
#![allow(dead_code)]

use libc::MSG_PEEK;

use crate::net::ToSocketAddrs;
use crate::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use crate::sys::fd::FileDesc;
use crate::sys::{AsInner, FromInner, IntoInner};

#[derive(Debug)]
pub struct Socket(FileDesc);

impl Socket {
    pub fn new(addr: &SocketAddr, ty: i32) -> io::Result<Socket> {
        unimplemented!()
    }

    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()> {
        unimplemented!()
    }

    pub fn connect_timeout(&self, addr: &SocketAddr, timeout: Duration) -> io::Result<()> {
        unimplemented!()
    }

    /*
    pub fn accept(
        &self,
        storage: *mut netc::sockaddr,
        len: *mut netc::socklen_t,
    ) -> io::Result<Socket> {
        unimplemented!()
    }
    */

    pub fn duplicate(&self) -> io::Result<Socket> {
        Ok(Self(self.0.duplicate()?))
    }

    fn recv_with_flags(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        unimplemented!()
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
        unimplemented!()
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
        unimplemented!()
    }

    pub fn timeout(&self, kind: i32) -> io::Result<Option<Duration>> {
        unimplemented!()
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        unimplemented!()
    }

    pub fn set_linger(&self, linger: Option<Duration>) -> io::Result<()> {
        unimplemented!()
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        unimplemented!()
    }

    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        unimplemented!()
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        unimplemented!()
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

    pub fn set_read_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        Ok(())
    }

    pub fn set_write_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        Ok(())
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        Ok(None)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        Ok(None)
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        todo!()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        todo!()
    }

    pub fn set_ttl(&self, _ttl: u32) -> io::Result<()> {
        todo!()
    }

    pub fn ttl(&self) -> io::Result<u32> {
        todo!()
    }

    pub fn set_only_v6(&self, _: bool) -> io::Result<()> {
        todo!()
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        todo!()
    }

    pub fn send_to(&self, _: &[u8], _: &SocketAddr) -> io::Result<usize> {
        unsupported()
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
        Self(file_desc)
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
        let socket = Socket::new(&a.to_socket_addrs()?.next().unwrap(), libc::SOCK_STREAM)?;
        Ok(Self(socket))
    }

    pub fn connect_timeout(a: &SocketAddr, d: Duration) -> io::Result<TcpStream> {
        // TODO: timeout
        let socket = Socket::new(&a, libc::SOCK_STREAM)?;
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

    pub fn bind<A: ToSocketAddrs>(_: A) -> io::Result<TcpListener> {
        unsupported()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0.socket_addr()
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        todo!()
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

    pub fn bind<A: ToSocketAddrs>(_: A) -> io::Result<UdpSocket> {
        unsupported()
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

    pub fn connect<A: ToSocketAddrs>(&self, a: A) -> io::Result<()> {
        todo!()
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

/*
#[allow(nonstandard_style)]
pub mod netc {
    pub const AF_INET: u8 = 0;
    pub const AF_INET6: u8 = 1;
    pub type sa_family_t = u8;

    #[derive(Copy, Clone)]
    pub struct in_addr {
        pub s_addr: u32,
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in {
        #[allow(dead_code)]
        pub sin_family: sa_family_t,
        pub sin_port: u16,
        pub sin_addr: in_addr,
    }

    #[derive(Copy, Clone)]
    pub struct in6_addr {
        pub s6_addr: [u8; 16],
    }

    #[derive(Copy, Clone)]
    pub struct sockaddr_in6 {
        #[allow(dead_code)]
        pub sin6_family: sa_family_t,
        pub sin6_port: u16,
        pub sin6_addr: in6_addr,
        pub sin6_flowinfo: u32,
        pub sin6_scope_id: u32,
    }
}
*/

pub fn lookup_host(_host: &str, _port: u16) -> io::Result<LookupHost> {
    unsupported()
}
