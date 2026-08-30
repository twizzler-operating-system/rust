//! Runtime interface for file descriptors.

use core::time::Duration;

pub use crate::bindings::descriptor as RawFd;
use crate::{
    error::{ArgumentError, RawTwzError, TwzError},
    nk, Result,
};

bitflags::bitflags! {
    /// Flags for file descriptors.
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct FdFlags : crate::bindings::fd_flags {
    /// The file descriptor refers to a terminal.
    const IS_TERMINAL = crate::bindings::FD_IS_TERMINAL;
}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
#[repr(u32)]
/// Possible Fd Kinds
pub enum FdKind {
    #[default]
    Regular = crate::bindings::fd_kind_FdKind_Regular,
    Directory = crate::bindings::fd_kind_FdKind_Directory,
    SymLink = crate::bindings::fd_kind_FdKind_SymLink,
    Pty = crate::bindings::fd_kind_FdKind_Pty,
    Socket = crate::bindings::fd_kind_FdKind_Socket,
    Pipe = crate::bindings::fd_kind_FdKind_Pipe,
    Compartment = crate::bindings::fd_kind_FdKind_Compartment,
    Other = u32::MAX,
}

impl From<u32> for FdKind {
    fn from(value: u32) -> Self {
        match value {
            crate::bindings::fd_kind_FdKind_Regular => Self::Regular,
            crate::bindings::fd_kind_FdKind_Directory => Self::Directory,
            crate::bindings::fd_kind_FdKind_SymLink => Self::SymLink,
            crate::bindings::fd_kind_FdKind_Pty => Self::Pty,
            crate::bindings::fd_kind_FdKind_Socket => Self::Socket,
            crate::bindings::fd_kind_FdKind_Pipe => Self::Pipe,
            crate::bindings::fd_kind_FdKind_Compartment => Self::Compartment,
            _ => Self::Other,
        }
    }
}

impl Into<u32> for FdKind {
    fn into(self) -> u32 {
        match self {
            Self::Regular => crate::bindings::fd_kind_FdKind_Regular,
            Self::Directory => crate::bindings::fd_kind_FdKind_Directory,
            Self::SymLink => crate::bindings::fd_kind_FdKind_SymLink,
            Self::Pty => crate::bindings::fd_kind_FdKind_Pty,
            Self::Socket => crate::bindings::fd_kind_FdKind_Socket,
            Self::Pipe => crate::bindings::fd_kind_FdKind_Pipe,
            Self::Compartment => crate::bindings::fd_kind_FdKind_Compartment,
            Self::Other => u32::MAX,
        }
    }
}

/// Information about an open file descriptor.
#[derive(Copy, Clone, Debug, Default)]
pub struct FdInfo {
    /// Flags for this descriptor
    pub flags: FdFlags,
    /// Length of underlying object
    pub size: u64,
    /// Kind of file
    pub kind: FdKind,
    /// Object ID
    pub id: twizzler_types::ObjID,
    /// Created time
    pub created: Duration,
    /// Accessed time
    pub accessed: Duration,
    /// Modified time
    pub modified: Duration,
    /// Unix mode
    pub unix_mode: u32,
}

impl From<crate::bindings::fd_info> for FdInfo {
    fn from(value: crate::bindings::fd_info) -> Self {
        Self {
            flags: FdFlags::from_bits_truncate(value.flags),
            size: value.len,
            kind: FdKind::from(value.kind),
            id: value.id,
            created: value.created.into(),
            accessed: value.accessed.into(),
            modified: value.modified.into(),
            unix_mode: value.unix_mode,
        }
    }
}

impl From<FdInfo> for crate::bindings::fd_info {
    fn from(value: FdInfo) -> Self {
        Self {
            flags: value.flags.bits(),
            len: value.size,
            kind: value.kind.into(),
            id: value.id,
            created: value.created.into(),
            accessed: value.accessed.into(),
            modified: value.modified.into(),
            unix_mode: value.unix_mode,
        }
    }
}

impl From<Result<RawFd>> for crate::bindings::open_result {
    fn from(value: Result<RawFd>) -> Self {
        match value {
            Ok(fd) => Self {
                fd,
                err: RawTwzError::success().raw(),
            },
            Err(e) => Self {
                fd: 0,
                err: e.raw(),
            },
        }
    }
}

impl From<crate::bindings::open_result> for Result<RawFd> {
    fn from(value: crate::bindings::open_result) -> Self {
        let raw = RawTwzError::new(value.err);
        if raw.is_success() {
            Ok(value.fd)
        } else {
            Err(raw.error())
        }
    }
}

/// Get information about an open file descriptor. If the fd is invalid or closed, returns None.
pub fn twz_rt_fd_get_info(fd: RawFd) -> Result<FdInfo> {
    let mut info = core::mem::MaybeUninit::uninit();
    unsafe {
        if nk!(crate::bindings::twz_rt_fd_get_info(fd, info.as_mut_ptr())) {
            return Ok(info.assume_init().into());
        }
    }
    Err(TwzError::Argument(ArgumentError::BadHandle))
}

pub use crate::bindings::name_entry as NameEntry;

impl NameEntry {
    pub const NAME_MAX_LEN: usize = crate::bindings::NAME_ENTRY_LEN as usize;
    pub fn new(iname: &[u8], info: crate::bindings::fd_info) -> Self {
        let nl = iname.len().min(Self::NAME_MAX_LEN);
        let mut name = [0; Self::NAME_MAX_LEN];
        name[0..nl].copy_from_slice(&iname[0..nl]);
        Self {
            name,
            info,
            name_len: nl as u32,
            linkname_len: 0,
        }
    }

    pub fn new_symlink(iname: &[u8], ilinkname: &[u8], info: crate::bindings::fd_info) -> Self {
        let nl = iname.len().min(Self::NAME_MAX_LEN);
        let linknl = ilinkname.len().min(Self::NAME_MAX_LEN - nl);
        let mut name = [0; Self::NAME_MAX_LEN];
        name[0..nl].copy_from_slice(&iname[0..nl]);
        name[nl..(nl + linknl)].copy_from_slice(&ilinkname[0..linknl]);
        Self {
            name,
            info,
            name_len: nl as u32,
            linkname_len: linknl as u32,
        }
    }

    pub fn name_bytes(&self) -> &[u8] {
        &self.name[0..self.name_len as usize]
    }

    pub fn linkname_bytes(&self) -> &[u8] {
        &self.name[self.name_len as usize..(self.name_len + self.linkname_len) as usize]
    }
}

/// Enumerate sub-names for an fd (e.g. directory entries). Returns Some(n) on success, None if no
/// names can be enumerated. Return of Some(n) indicates number of items read into the buffer, 0 if
/// end of name list. Offset argument specifies number of entries to skip.
pub fn twz_rt_fd_enumerate_names(
    fd: RawFd,
    entries: &mut [NameEntry],
    off: usize,
) -> Result<usize> {
    unsafe {
        nk!(crate::bindings::twz_rt_fd_enumerate_names(
            fd,
            entries.as_mut_ptr(),
            entries.len(),
            off
        )
        .into())
    }
}

use crate::bindings::{
    binding_info, descriptor, object_bind_info, objid, prot_kind, socket_address, socket_bind_info,
};
impl binding_info {
    pub fn new_object_binding(fd: descriptor, kind: OpenKind, flags: u32, id: objid) -> Self {
        let mut this = Self {
            kind: kind.into(),
            flags,
            fd,
            bind_data: [0; _],
            bind_len: size_of::<object_bind_info>() as u32,
        };
        let bind_info = object_bind_info { id };
        let bind_info_bytes = &bind_info as *const _ as *const u8;
        let slice =
            unsafe { core::slice::from_raw_parts(bind_info_bytes, size_of::<object_bind_info>()) };
        this.bind_data[0..(this.bind_len as usize)].copy_from_slice(slice);
        this
    }

    pub fn new_socket_binding(
        fd: descriptor,
        kind: OpenKind,
        flags: u32,
        addr: socket_address,
        prot: prot_kind,
    ) -> Self {
        let mut this = Self {
            kind: kind.into(),
            flags,
            fd,
            bind_data: [0; _],
            bind_len: size_of::<socket_bind_info>() as u32,
        };
        let bind_info = socket_bind_info { addr, prot };
        let bind_info_bytes = &bind_info as *const _ as *const u8;
        let slice =
            unsafe { core::slice::from_raw_parts(bind_info_bytes, size_of::<socket_bind_info>()) };
        this.bind_data[0..(this.bind_len as usize)].copy_from_slice(slice);
        this
    }

    pub fn new_fd_binding(fd: descriptor, kind: OpenKind, flags: u32, bind_fd: descriptor) -> Self {
        let mut this = Self {
            kind: kind.into(),
            flags,
            fd,
            bind_data: [0; _],
            bind_len: size_of::<descriptor>() as u32,
        };
        let bind_info_bytes = &bind_fd as *const _ as *const u8;
        let slice =
            unsafe { core::slice::from_raw_parts(bind_info_bytes, size_of::<descriptor>()) };
        this.bind_data[0..(this.bind_len as usize)].copy_from_slice(slice);
        this
    }
}

pub fn twz_rt_fd_read_binds(binds: &mut [binding_info]) -> usize {
    unsafe {
        nk!(crate::bindings::twz_rt_fd_read_binds(
            binds.as_mut_ptr(),
            binds.len()
        ))
    }
}

/// Open a file descriptor by name, as a C-string.
pub fn twz_rt_fd_copen(
    name: &core::ffi::CStr,
    create: crate::bindings::create_options,
    flags: u32,
) -> Result<RawFd> {
    let name_len = name.count_bytes().min(crate::bindings::NAME_DATA_MAX);
    let mut info = crate::bindings::open_info {
        len: name_len,
        create,
        flags,
        name: [0; _],
    };
    info.name[0..name_len].copy_from_slice(&name.to_bytes()[0..name_len]);
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            crate::bindings::open_kind_OpenKind_Path,
            flags,
            (&mut info as *mut crate::bindings::open_info).cast(),
            size_of_val(&info),
        )
        .into())
    }
}

/// Open a file descriptor by name, as a Rust-string.
pub fn twz_rt_fd_open(
    name: &str,
    create: crate::bindings::create_options,
    flags: u32,
) -> Result<RawFd> {
    let name_len = name.as_bytes().len().min(crate::bindings::NAME_DATA_MAX);
    let mut info = crate::bindings::open_info {
        len: name_len,
        create,
        flags,
        name: [0; _],
    };
    info.name[0..name_len].copy_from_slice(&name.as_bytes()[0..name_len]);
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            crate::bindings::open_kind_OpenKind_Path,
            flags,
            (&mut info as *mut crate::bindings::open_info).cast(),
            size_of_val(&info),
        )
        .into())
    }
}

/// Remove a name
pub fn twz_rt_fd_remove(name: &str) -> Result<()> {
    unsafe {
        RawTwzError::new(nk!(crate::bindings::twz_rt_fd_remove(
            name.as_ptr().cast(),
            name.len(),
        )))
        .result()
    }
}

/// Make a new namespace
pub fn twz_rt_fd_mkns(name: &str) -> Result<()> {
    unsafe {
        RawTwzError::new(nk!(crate::bindings::twz_rt_fd_mkns(
            name.as_ptr().cast(),
            name.len(),
        )))
        .result()
    }
}

/// Make a new symlink
pub fn twz_rt_fd_symlink(name: &str, target: &str) -> Result<()> {
    unsafe {
        RawTwzError::new(nk!(crate::bindings::twz_rt_fd_symlink(
            name.as_ptr().cast(),
            name.len(),
            target.as_ptr().cast(),
            target.len(),
        )))
        .result()
    }
}

/// Rename a name in the namespace.
pub fn twz_rt_fd_rename(old_name: &str, new_name: &str) -> Result<()> {
    unsafe {
        RawTwzError::new(nk!(crate::bindings::twz_rt_fd_rename(
            old_name.as_ptr().cast(),
            old_name.len(),
            new_name.as_ptr().cast(),
            new_name.len(),
        )))
        .result()
    }
}

pub fn twz_rt_fd_readlink(name: &str, buf: &mut [u8]) -> Result<usize> {
    let mut len: u64 = 0;
    unsafe {
        RawTwzError::new(nk!(crate::bindings::twz_rt_fd_readlink(
            name.as_ptr().cast(),
            name.len(),
            buf.as_mut_ptr().cast(),
            buf.len(),
            &mut len,
        )))
        .result()?;
    }
    Ok(len as usize)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum OpenKind {
    Object = crate::bindings::open_kind_OpenKind_Object,
    Path = crate::bindings::open_kind_OpenKind_Path,
    Pipe = crate::bindings::open_kind_OpenKind_Pipe,
    SocketConnect = crate::bindings::open_kind_OpenKind_SocketConnect,
    SocketBind = crate::bindings::open_kind_OpenKind_SocketBind,
    SocketAccept = crate::bindings::open_kind_OpenKind_SocketAccept,
    PtyServer = crate::bindings::open_kind_OpenKind_PtyServer,
    PtyClient = crate::bindings::open_kind_OpenKind_PtyClient,
    Compartment = crate::bindings::open_kind_OpenKind_Compartment,
    KernelConsole = crate::bindings::open_kind_OpenKind_KernelConsole,
    Kqueue = crate::bindings::open_kind_OpenKind_Kqueue,
}

impl TryFrom<u32> for OpenKind {
    type Error = ();

    fn try_from(val: u32) -> core::result::Result<OpenKind, Self::Error> {
        match val {
            crate::bindings::open_kind_OpenKind_Pipe => Ok(Self::Pipe),
            crate::bindings::open_kind_OpenKind_SocketConnect => Ok(Self::SocketConnect),
            crate::bindings::open_kind_OpenKind_SocketBind => Ok(Self::SocketBind),
            crate::bindings::open_kind_OpenKind_SocketAccept => Ok(Self::SocketAccept),
            crate::bindings::open_kind_OpenKind_PtyServer => Ok(Self::PtyServer),
            crate::bindings::open_kind_OpenKind_PtyClient => Ok(Self::PtyClient),
            crate::bindings::open_kind_OpenKind_Path => Ok(Self::Path),
            crate::bindings::open_kind_OpenKind_Object => Ok(Self::Object),
            crate::bindings::open_kind_OpenKind_Compartment => Ok(Self::Compartment),
            crate::bindings::open_kind_OpenKind_KernelConsole => Ok(Self::KernelConsole),
            crate::bindings::open_kind_OpenKind_Kqueue => Ok(Self::Kqueue),

            _ => Err(()),
        }
    }
}

impl From<OpenKind> for u32 {
    fn from(val: OpenKind) -> u32 {
        match val {
            OpenKind::Pipe => crate::bindings::open_kind_OpenKind_Pipe,
            OpenKind::Path => crate::bindings::open_kind_OpenKind_Path,
            OpenKind::Object => crate::bindings::open_kind_OpenKind_Object,
            OpenKind::SocketConnect => crate::bindings::open_kind_OpenKind_SocketConnect,
            OpenKind::SocketBind => crate::bindings::open_kind_OpenKind_SocketBind,
            OpenKind::SocketAccept => crate::bindings::open_kind_OpenKind_SocketAccept,
            OpenKind::PtyServer => crate::bindings::open_kind_OpenKind_PtyServer,
            OpenKind::PtyClient => crate::bindings::open_kind_OpenKind_PtyClient,
            OpenKind::Compartment => crate::bindings::open_kind_OpenKind_Compartment,
            OpenKind::KernelConsole => crate::bindings::open_kind_OpenKind_KernelConsole,
            OpenKind::Kqueue => crate::bindings::open_kind_OpenKind_Kqueue,
        }
    }
}

#[derive(Clone, Copy, Default)]
#[repr(transparent)]
pub struct SocketAddress(pub crate::bindings::socket_address);

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(u32)]
pub enum ProtKind {
    Stream = crate::bindings::prot_kind_ProtKind_Stream,
    Datagram = crate::bindings::prot_kind_ProtKind_Datagram,
}

impl SocketAddress {
    fn new_v4(octets: [u8; 4], port: u16) -> Self {
        Self(crate::bindings::socket_address {
            kind: crate::bindings::addr_kind_AddrKind_Ipv4,
            addr_octets: crate::bindings::socket_address_addrs { v4: octets },
            port,
            flowinfo: 0,
            scope_id: 0,
        })
    }

    fn new_v6(octets: [u8; 16], port: u16, flowinfo: u32, scope_id: u32) -> Self {
        Self(crate::bindings::socket_address {
            kind: crate::bindings::addr_kind_AddrKind_Ipv6,
            addr_octets: crate::bindings::socket_address_addrs { v6: octets },
            port,
            flowinfo,
            scope_id,
        })
    }

    fn v4_octets(&self) -> [u8; 4] {
        assert!(self.is_v4());
        unsafe { self.0.addr_octets.v4 }
    }

    fn v6_octets(&self) -> [u8; 16] {
        assert!(!self.is_v4());
        unsafe { self.0.addr_octets.v6 }
    }

    fn is_v4(&self) -> bool {
        self.0.kind == crate::bindings::addr_kind_AddrKind_Ipv4
    }
}

impl From<SocketAddress> for core::net::IpAddr {
    fn from(value: SocketAddress) -> Self {
        if value.is_v4() {
            Self::V4(core::net::Ipv4Addr::from_octets(value.v4_octets()))
        } else {
            Self::V6(core::net::Ipv6Addr::from_octets(value.v6_octets()))
        }
    }
}

impl From<SocketAddress> for core::net::SocketAddr {
    fn from(value: SocketAddress) -> Self {
        if value.is_v4() {
            Self::V4(core::net::SocketAddrV4::new(
                core::net::Ipv4Addr::from_octets(value.v4_octets()),
                value.0.port,
            ))
        } else {
            Self::V6(core::net::SocketAddrV6::new(
                core::net::Ipv6Addr::from_octets(value.v6_octets()),
                value.0.port,
                value.0.flowinfo,
                value.0.scope_id,
            ))
        }
    }
}

impl From<core::net::Ipv4Addr> for SocketAddress {
    fn from(value: core::net::Ipv4Addr) -> Self {
        Self::new_v4(value.octets(), 0)
    }
}

impl From<core::net::Ipv6Addr> for SocketAddress {
    fn from(value: core::net::Ipv6Addr) -> Self {
        Self::new_v6(value.octets(), 0, 0, 0)
    }
}

impl From<core::net::SocketAddrV4> for SocketAddress {
    fn from(value: core::net::SocketAddrV4) -> Self {
        Self::new_v4(value.ip().octets(), value.port())
    }
}

impl From<core::net::SocketAddrV6> for SocketAddress {
    fn from(value: core::net::SocketAddrV6) -> Self {
        Self::new_v6(
            value.ip().octets(),
            value.port(),
            value.flowinfo(),
            value.scope_id(),
        )
    }
}

impl From<core::net::SocketAddr> for SocketAddress {
    fn from(value: core::net::SocketAddr) -> Self {
        match value {
            core::net::SocketAddr::V4(v4) => v4.into(),
            core::net::SocketAddr::V6(v6) => v6.into(),
        }
    }
}

impl From<core::net::IpAddr> for SocketAddress {
    fn from(value: core::net::IpAddr) -> Self {
        match value {
            core::net::IpAddr::V4(v4) => v4.into(),
            core::net::IpAddr::V6(v6) => v6.into(),
        }
    }
}

/// Open an anonymous file descriptor.
pub fn twz_rt_fd_open_socket_bind(
    addr: SocketAddress,
    flags: u32,
    prot: ProtKind,
) -> Result<RawFd> {
    let mut binding = crate::bindings::socket_bind_info {
        addr: addr.0,
        prot: prot as u32,
    };
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            OpenKind::SocketBind.into(),
            flags,
            ((&mut binding) as *mut crate::bindings::socket_bind_info).cast(),
            core::mem::size_of::<crate::bindings::socket_bind_info>(),
        ))
        .into()
    }
}

/// Open an anonymous file descriptor.
pub fn twz_rt_fd_open_socket(flags: u32, _prot: ProtKind) -> Result<RawFd> {
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            OpenKind::SocketBind.into(),
            flags,
            core::ptr::null_mut(),
            0,
        ))
        .into()
    }
}

/// Open an anonymous file descriptor.
pub fn twz_rt_fd_socket_rebind(
    fd: RawFd,
    addr: SocketAddress,
    flags: u32,
    prot: ProtKind,
) -> Result<()> {
    let mut binding = crate::bindings::socket_bind_info {
        addr: addr.0,
        prot: prot as u32,
    };
    unsafe {
        RawTwzError::new(nk!(crate::bindings::twz_rt_fd_reopen(
            fd,
            OpenKind::SocketBind.into(),
            flags,
            ((&mut binding) as *mut crate::bindings::socket_bind_info).cast(),
            core::mem::size_of::<crate::bindings::socket_bind_info>(),
        )))
        .result()
    }
}

// Accept a connection on a bound socket file descriptor, creating a new file descriptor.
pub fn twz_rt_fd_open_socket_accept(fd: RawFd, flags: u32) -> Result<RawFd> {
    let mut binding = crate::bindings::object_bind_info { id: fd as objid };
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            OpenKind::SocketAccept.into(),
            flags,
            ((&mut binding) as *mut crate::bindings::object_bind_info).cast(),
            core::mem::size_of::<crate::bindings::object_bind_info>(),
        ))
        .into()
    }
}

/// Open an anonymous file descriptor.
pub fn twz_rt_fd_open_socket_connect(
    addr: SocketAddress,
    flags: u32,
    prot: ProtKind,
) -> Result<RawFd> {
    let mut binding = crate::bindings::socket_bind_info {
        addr: addr.0,
        prot: prot as u32,
    };
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            OpenKind::SocketConnect.into(),
            flags,
            ((&mut binding) as *mut crate::bindings::socket_bind_info).cast(),
            core::mem::size_of::<crate::bindings::socket_bind_info>(),
        ))
        .into()
    }
}

/// Open an anonymous file descriptor.
pub fn twz_rt_fd_socket_reconnect(
    fd: RawFd,
    addr: SocketAddress,
    flags: u32,
    prot: ProtKind,
) -> Result<()> {
    let mut binding = crate::bindings::socket_bind_info {
        addr: addr.0,
        prot: prot as u32,
    };
    unsafe {
        RawTwzError::new(nk!(crate::bindings::twz_rt_fd_reopen(
            fd,
            OpenKind::SocketConnect.into(),
            flags,
            ((&mut binding) as *mut crate::bindings::socket_bind_info).cast(),
            core::mem::size_of::<crate::bindings::socket_bind_info>(),
        )))
        .result()
    }
}

/// Open an PTY.
pub fn twz_rt_fd_open_pty_server(id: twizzler_types::ObjID, flags: u32) -> Result<RawFd> {
    let mut binding = crate::bindings::object_bind_info { id };
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            OpenKind::PtyServer.into(),
            flags,
            ((&mut binding) as *mut crate::bindings::object_bind_info).cast(),
            core::mem::size_of::<crate::bindings::object_bind_info>(),
        ))
        .into()
    }
}

/// Open an PTY.
pub fn twz_rt_fd_open_pty_client(id: twizzler_types::ObjID, flags: u32) -> Result<RawFd> {
    let mut binding = crate::bindings::object_bind_info { id };
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            OpenKind::PtyClient.into(),
            flags,
            ((&mut binding) as *mut crate::bindings::object_bind_info).cast(),
            core::mem::size_of::<crate::bindings::object_bind_info>(),
        ))
        .into()
    }
}

/// Open an anonymous file descriptor.
pub fn twz_rt_fd_open_compartment(id: crate::bindings::objid, flags: u32) -> Result<RawFd> {
    let mut binding = crate::bindings::object_bind_info { id };
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            OpenKind::Compartment.into(),
            flags,
            ((&mut binding) as *mut crate::bindings::object_bind_info).cast(),
            core::mem::size_of::<crate::bindings::object_bind_info>(),
        ))
        .into()
    }
}

/// Open an anonymous file descriptor.
pub fn twz_rt_fd_open_pipe(id: Option<crate::bindings::objid>, flags: u32) -> Result<RawFd> {
    let mut binding = crate::bindings::object_bind_info {
        id: id.unwrap_or(0),
    };
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            OpenKind::Pipe.into(),
            flags,
            ((&mut binding) as *mut crate::bindings::object_bind_info).cast(),
            core::mem::size_of::<crate::bindings::object_bind_info>(),
        ))
        .into()
    }
}

/// Create a new kqueue file descriptor.
pub fn twz_rt_fd_open_kqueue(flags: u32) -> Result<RawFd> {
    unsafe {
        nk!(crate::bindings::twz_rt_fd_open(
            OpenKind::Kqueue.into(),
            flags,
            core::ptr::null_mut(),
            0,
        ))
        .into()
    }
}

/// Duplicate a file descriptor.
pub fn twz_rt_fd_dup(fd: RawFd) -> Result<RawFd> {
    let mut new_fd = core::mem::MaybeUninit::<RawFd>::uninit();
    unsafe {
        let e = nk!(crate::bindings::twz_rt_fd_cmd(
            fd,
            crate::bindings::FD_CMD_DUP,
            core::ptr::null_mut(),
            new_fd.as_mut_ptr().cast(),
        ));
        let raw = RawTwzError::new(e);
        if raw.is_success() {
            Ok(new_fd.assume_init())
        } else {
            Err(raw.error())
        }
    }
}

/// Duplicate a file descriptor onto a specific descriptor number, closing whatever `to` referred
/// to beforehand. Duplicating a descriptor onto itself succeeds and does nothing.
pub fn twz_rt_fd_dup2(fd: RawFd, to: RawFd) -> Result<RawFd> {
    let mut new_fd = core::mem::MaybeUninit::<RawFd>::uninit();
    let mut to = to;
    unsafe {
        let e = nk!(crate::bindings::twz_rt_fd_cmd(
            fd,
            crate::bindings::FD_CMD_DUP2,
            (&raw mut to).cast(),
            new_fd.as_mut_ptr().cast(),
        ));
        let raw = RawTwzError::new(e);
        if raw.is_success() {
            Ok(new_fd.assume_init())
        } else {
            Err(raw.error())
        }
    }
}

/// Read a descriptor's close-on-exec flag.
pub fn twz_rt_fd_get_cloexec(fd: RawFd) -> Result<bool> {
    let mut val: u32 = 0;
    unsafe {
        let e = nk!(crate::bindings::twz_rt_fd_cmd(
            fd,
            crate::bindings::FD_CMD_GET_CLOEXEC,
            core::ptr::null_mut(),
            (&raw mut val).cast(),
        ));
        let raw = RawTwzError::new(e);
        if !raw.is_success() {
            return Err(raw.error());
        }
    }
    Ok(val != 0)
}

/// Set or clear a descriptor's close-on-exec flag. A descriptor made by [twz_rt_fd_dup] or
/// [twz_rt_fd_dup2] always starts with the flag clear.
pub fn twz_rt_fd_set_cloexec(fd: RawFd, on: bool) -> Result<()> {
    let mut val: u32 = on as u32;
    unsafe {
        let e = nk!(crate::bindings::twz_rt_fd_cmd(
            fd,
            crate::bindings::FD_CMD_SET_CLOEXEC,
            (&raw mut val).cast(),
            core::ptr::null_mut(),
        ));
        let raw = RawTwzError::new(e);
        if !raw.is_success() {
            return Err(raw.error());
        }
    }
    Ok(())
}

/// Sync a file descriptor.
pub fn twz_rt_fd_sync(fd: RawFd) {
    unsafe {
        nk!(crate::bindings::twz_rt_fd_cmd(
            fd,
            crate::bindings::FD_CMD_SYNC,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ));
    }
}

/// Truncate a file descriptor.
pub fn twz_rt_fd_truncate(fd: RawFd, mut len: u64) -> Result<()> {
    unsafe {
        let e = nk!(crate::bindings::twz_rt_fd_cmd(
            fd,
            crate::bindings::FD_CMD_TRUNCATE,
            (&mut len as *mut u64).cast(),
            core::ptr::null_mut(),
        ));
        let raw = RawTwzError::new(e);
        if !raw.is_success() {
            return Err(raw.error());
        }
    }
    Ok(())
}

pub fn twz_rt_fd_shutdown(fd: RawFd, read: bool, write: bool) -> Result<()> {
    let mut bits: u32 = 0;
    if read {
        bits |= 1;
    }
    if write {
        bits |= 2;
    }
    unsafe {
        let e = nk!(crate::bindings::twz_rt_fd_cmd(
            fd,
            crate::bindings::FD_CMD_SHUTDOWN,
            (&mut bits as *mut u32).cast(),
            core::ptr::null_mut(),
        ));
        let raw = RawTwzError::new(e);
        if !raw.is_success() {
            return Err(raw.error());
        }
    }
    Ok(())
}

/// Close a file descriptor. If the fd is already closed, or invalid, this function has no effect.
pub fn twz_rt_fd_close(fd: RawFd) {
    unsafe { nk!(crate::bindings::twz_rt_fd_close(fd)) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(u32)]
pub enum NameRoot {
    Root = crate::bindings::name_root_NameRoot_Root,
    Home = crate::bindings::name_root_NameRoot_Home,
    Current = crate::bindings::name_root_NameRoot_Current,
    Temp = crate::bindings::name_root_NameRoot_Temp,
    Exe = crate::bindings::name_root_NameRoot_Exe,
}

impl From<u32> for NameRoot {
    fn from(value: u32) -> Self {
        match value {
            crate::bindings::name_root_NameRoot_Root => NameRoot::Root,
            crate::bindings::name_root_NameRoot_Home => NameRoot::Home,
            crate::bindings::name_root_NameRoot_Current => NameRoot::Current,
            crate::bindings::name_root_NameRoot_Temp => NameRoot::Temp,
            crate::bindings::name_root_NameRoot_Exe => NameRoot::Exe,
            _ => panic!("invalid NameRoot value"),
        }
    }
}

pub fn twz_rt_get_nameroot(root: NameRoot, buf: &mut [u8]) -> Result<usize> {
    unsafe {
        nk!(
            crate::bindings::twz_rt_get_nameroot(root as u32, buf.as_mut_ptr().cast(), buf.len())
                .into()
        )
    }
}

pub fn twz_rt_set_nameroot(root: NameRoot, buf: &[u8]) -> Result<()> {
    let res = unsafe {
        nk!(crate::bindings::twz_rt_set_nameroot(
            root as u32,
            buf.as_ptr().cast(),
            buf.len()
        ))
    };
    let r = RawTwzError::new(res);
    if r.is_success() {
        Ok(())
    } else {
        Err(r.error())
    }
}

#[derive(Default, Copy, Clone)]
#[repr(u32)]
pub enum NameResolver {
    #[default]
    Default = crate::bindings::name_resolver_NameResolver_Default,
    Socket = crate::bindings::name_resolver_NameResolver_Socket,
}

impl From<u32> for NameResolver {
    fn from(value: u32) -> Self {
        match value {
            crate::bindings::name_resolver_NameResolver_Default => NameResolver::Default,
            crate::bindings::name_resolver_NameResolver_Socket => NameResolver::Socket,
            _ => panic!("invalid NameResolver value"),
        }
    }
}

pub fn twz_rt_resolve_name(
    resolver: NameResolver,
    name: impl AsRef<str>,
) -> Result<crate::object::ObjID> {
    let name = name.as_ref().as_bytes();
    let res = unsafe {
        nk!(crate::bindings::twz_rt_resolve_name(
            resolver as u32,
            name.as_ptr().cast(),
            name.len()
        ))
    };
    let r = RawTwzError::new(res.err);
    if r.is_success() {
        Ok(res.val.into())
    } else {
        Err(r.error())
    }
}

pub fn twz_rt_canon_name(
    resolver: NameResolver,
    name: impl AsRef<str>,
    out_name: &mut [u8],
) -> Result<usize> {
    let name = name.as_ref().as_bytes();
    let mut out_len = out_name.len();
    let res = unsafe {
        nk!(crate::bindings::twz_rt_canon_name(
            resolver as u32,
            name.as_ptr().cast(),
            name.len(),
            out_name.as_mut_ptr().cast(),
            &mut out_len,
        ))
    };
    let r = RawTwzError::new(res);
    if r.is_success() {
        Ok(out_len)
    } else {
        Err(r.error())
    }
}

pub fn twz_rt_socket_names(
    name: impl AsRef<str>,
    out_addrs: &mut [SocketAddress],
) -> Result<usize> {
    let name = name.as_ref().as_bytes();
    let mut out_len = out_addrs.len() * size_of::<SocketAddress>();
    let res = unsafe {
        nk!(crate::bindings::twz_rt_canon_name(
            NameResolver::Socket as u32,
            name.as_ptr().cast(),
            name.len(),
            out_addrs.as_mut_ptr().cast(),
            &mut out_len,
        ))
    };
    let r = RawTwzError::new(res);
    if r.is_success() {
        Ok(out_len / size_of::<SocketAddress>())
    } else {
        Err(r.error())
    }
}
