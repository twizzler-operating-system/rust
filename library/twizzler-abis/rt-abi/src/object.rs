//! Interface for objects and object handles.

use core::{
    ffi::c_void,
    fmt::{LowerHex, UpperHex},
    mem::MaybeUninit,
    ptr::{addr_of, addr_of_mut},
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

use bitflags::bitflags;

use crate::{
    bindings::{
        object_cmd, object_create, object_tie, sync_info, twz_rt_object_cmd, LEN_MUL,
        OBJECT_CMD_DELETE, OBJECT_CMD_SYNC, OBJECT_CMD_UPDATE,
    },
    error::{RawTwzError, ResourceError, TwzError},
    nk, Result,
};

/// An object ID.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
#[repr(transparent)]
pub struct ObjID(twizzler_types::ObjID);

impl ObjID {
    /// The number of u64 components that make up an object ID, if split.
    pub const NR_PARTS: usize = 2;

    /// Build a new object ID from raw.
    pub const fn new(raw: twizzler_types::ObjID) -> Self {
        Self(raw)
    }

    /// Get the raw object ID type.
    pub const fn raw(&self) -> twizzler_types::ObjID {
        self.0
    }

    /// Build an object ID from parts, useful for syscalls.
    pub const fn from_parts(parts: [u64; Self::NR_PARTS]) -> Self {
        Self::new(((parts[0] as u128) << 64) | (parts[1] as u128))
    }

    /// Split the object ID into parts, useful for packing into registers for syscalls.
    pub const fn parts(&self) -> [u64; Self::NR_PARTS] {
        [(self.0 >> 64) as u64, (self.0 & 0xffffffffffffffff) as u64]
    }
}

impl core::convert::AsRef<ObjID> for ObjID {
    fn as_ref(&self) -> &ObjID {
        self
    }
}

impl From<twizzler_types::ObjID> for ObjID {
    fn from(id: twizzler_types::ObjID) -> Self {
        Self::new(id)
    }
}

impl LowerHex for ObjID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl UpperHex for ObjID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

impl core::fmt::Display for ObjID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ObjID({:x})", self.0)
    }
}

impl core::fmt::Debug for ObjID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ObjID({:x})", self.0)
    }
}

/// An object handle, granting access to object memory. An object handle can be in two modes:
///   - Owning -- the normal mode, which acts like an Arc, and asks the runtime to unmap when
///     refcount hits zero.
///   - Unsafe -- internal use only, is NOT owning, but still has pointers. This is totally unsafe
///     to use, and should not be exposed to users. But sometimes, it can be safe, and faster than
///     cloning.
/// ... anyway, in general these have reference counting semantics, via Clone and Drop, like Arc.
#[repr(transparent)]
pub struct ObjectHandle(pub(crate) crate::bindings::object_handle);

impl core::fmt::Debug for ObjectHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ObjectHandle({:?}, {:p}, {:x}, {:?})",
            self.id(),
            self.start(),
            self.valid_len(),
            self.map_flags()
        )
    }
}

unsafe impl Send for ObjectHandle {}
unsafe impl Sync for ObjectHandle {}

bitflags::bitflags! {
    /// Flags for mapping objects.
    #[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Debug)]
    pub struct MapFlags : crate::bindings::map_flags {
        /// Request READ access.
        const READ = crate::bindings::MAP_FLAG_R;
        /// Request WRITE access.
        const WRITE = crate::bindings::MAP_FLAG_W;
        /// Request EXECUTE access.
        const EXEC = crate::bindings::MAP_FLAG_X;
        /// Persist changes on flush.
        const PERSIST = crate::bindings::MAP_FLAG_PERSIST;
        /// Use runtime support for read stability.
        const INDIRECT = crate::bindings::MAP_FLAG_INDIRECT;
    }
}

impl MapFlags {
    pub fn rw() -> Self {
        Self::READ | Self::WRITE | Self::PERSIST
    }

    pub fn rw_volatile() -> Self {
        Self::READ | Self::WRITE
    }

    pub fn ro() -> Self {
        Self::READ | Self::INDIRECT
    }

    pub fn rx() -> Self {
        Self::READ | Self::EXEC | Self::INDIRECT
    }
}

#[repr(u32)]
pub enum ObjectCmd {
    Delete = OBJECT_CMD_DELETE,
    Sync = OBJECT_CMD_SYNC,
    Update = OBJECT_CMD_UPDATE,
}

impl TryFrom<object_cmd> for ObjectCmd {
    type Error = TwzError;

    fn try_from(value: object_cmd) -> core::result::Result<Self, Self::Error> {
        match value {
            OBJECT_CMD_DELETE => Ok(ObjectCmd::Delete),
            OBJECT_CMD_SYNC => Ok(ObjectCmd::Sync),
            OBJECT_CMD_UPDATE => Ok(ObjectCmd::Update),
            _ => Err(TwzError::INVALID_ARGUMENT),
        }
    }
}

#[allow(dead_code)]
impl ObjectHandle {
    fn refs(&self) -> *const AtomicU64 {
        self.0.runtime_info.cast()
    }

    /// Get a pointer to the start of object data.
    pub fn start(&self) -> *mut u8 {
        self.0.start.cast()
    }

    /// Get a pointer to the metadata structure.
    pub fn meta(&self) -> *mut MetaInfo {
        self.0.meta.cast()
    }

    /// Get a slice of metadata extensions
    pub fn meta_exts(&self) -> &[MetaExt] {
        unsafe {
            core::slice::from_raw_parts(
                self.0.meta.cast::<u8>().add(size_of::<MetaInfo>()).cast(),
                (*self.meta()).extcount as usize,
            )
        }
    }

    /// Get a slice of metadata extensions
    fn all_meta_exts(&self) -> &[MetaExt] {
        unsafe {
            core::slice::from_raw_parts(
                self.0.meta.cast::<u8>().add(size_of::<MetaInfo>()).cast(),
                16,
            )
        }
    }

    /// Find the first metadata extension matching the given tag
    pub fn find_meta_ext(&self, tag: MetaExtTag) -> Option<&MetaExt> {
        self.meta_exts()
            .iter()
            .find(|e| e.value.load(Ordering::SeqCst) != 0 && e.tag == tag)
    }

    pub unsafe fn set_meta_ext(&self, ext: MetaExt) -> Result<()> {
        let meta_exts = self.meta_exts();
        for me in meta_exts {
            if me.tag == ext.tag {
                me.value
                    .store(ext.value.load(Ordering::SeqCst), Ordering::SeqCst);
                return Ok(());
            }
        }
        let meta_exts = self.all_meta_exts();
        for (idx, me) in meta_exts.iter().enumerate() {
            if me.tag == MEXT_EMPTY {
                if me
                    .value
                    .swap(ext.value.load(Ordering::Relaxed), Ordering::SeqCst)
                    == 0
                {
                    let ptr = addr_of!(me.tag) as *mut MetaExtTag;
                    ptr.write(ext.tag);
                    (*self.meta()).extcount = (idx + 1) as u16;
                    return Ok(());
                }
            }
        }
        Err(TwzError::Resource(ResourceError::OutOfResources))
    }

    /// Get a pointer to the runtime info.
    pub fn runtime_info(&self) -> *mut u8 {
        self.0.runtime_info.cast()
    }

    /// Get map flags.
    pub fn map_flags(&self) -> MapFlags {
        MapFlags::from_bits_truncate(self.0.map_flags)
    }

    /// Get the number of valid bytes after start pointer for object data.
    pub fn valid_len(&self) -> usize {
        (self.0.valid_len as usize) * crate::bindings::LEN_MUL
    }

    /// Get the object ID.
    pub fn id(&self) -> ObjID {
        ObjID::new(self.0.id)
    }

    /// Get the raw object handle.
    pub fn into_raw(self) -> crate::bindings::object_handle {
        let this = core::mem::ManuallyDrop::new(self);
        this.0
    }

    /// Build an object handle from raw.
    pub fn from_raw(raw: crate::bindings::object_handle) -> Self {
        Self(raw)
    }

    /// Modify an object.
    pub fn cmd<T>(&self, cmd: ObjectCmd, arg: *mut T) -> Result<()> {
        let err = unsafe {
            nk!(twz_rt_object_cmd(
                &self.0 as *const _ as *mut _,
                cmd as object_cmd,
                arg.cast()
            ))
        };
        let raw = RawTwzError::new(err);
        if raw.is_success() {
            Ok(())
        } else {
            Err(raw.error())
        }
    }

    /// Make a new object handle.
    ///
    /// # Safety
    /// The caller must ensure that runtime_info is a valid pointer, and points to a repr(C) struct
    /// that starts with an AtomicU64 for reference counting.
    pub unsafe fn new(
        id: ObjID,
        runtime_info: *mut core::ffi::c_void,
        start: *mut core::ffi::c_void,
        meta: *mut core::ffi::c_void,
        map_flags: MapFlags,
        valid_len: usize,
    ) -> Self {
        Self::from_raw(crate::bindings::object_handle {
            id: id.0,
            runtime_info,
            start,
            meta,
            map_flags: map_flags.bits(),
            valid_len: (valid_len / LEN_MUL) as u32,
        })
    }
}

impl Clone for ObjectHandle {
    fn clone(&self) -> Self {
        unsafe {
            let Some(ref rc) = self.refs().as_ref() else {
                return Self(self.0);
            };
            // This use of Relaxed ordering is justified by https://doc.rust-lang.org/nomicon/arc-mutex/arc-clone.html.
            let old_count = rc.fetch_add(1, Ordering::Relaxed);
            // The above link also justifies the following behavior. If our count gets this high, we
            // have probably run into a problem somewhere.
            if old_count >= i64::MAX as u64 {
                nk!(crate::core::twz_rt_abort());
            }
        }
        Self(self.0)
    }
}

impl Drop for ObjectHandle {
    fn drop(&mut self) {
        unsafe {
            let Some(ref rc) = self.refs().as_ref() else {
                return;
            };
            // This use of Release ordering is justified by https://doc.rust-lang.org/nomicon/arc-mutex/arc-clone.html.
            if rc.fetch_sub(1, Ordering::Release) != 1 {
                return;
            }
        }
        // This fence is needed to prevent reordering of the use and deletion
        // of the data.
        core::sync::atomic::fence(Ordering::Acquire);
        nk!(twz_rt_release_handle(self, 0));
    }
}

impl AsRef<ObjectHandle> for ObjectHandle {
    fn as_ref(&self) -> &ObjectHandle {
        self
    }
}

impl From<Result<ObjectHandle>> for crate::bindings::map_result {
    fn from(value: Result<ObjectHandle>) -> Self {
        match value {
            Ok(handle) => Self {
                handle: handle.into_raw(),
                error: RawTwzError::success().raw(),
            },
            Err(e) => Self {
                handle: crate::bindings::object_handle::default(),
                error: e.raw(),
            },
        }
    }
}

impl From<crate::bindings::map_result> for Result<ObjectHandle> {
    fn from(value: crate::bindings::map_result) -> Self {
        let raw = RawTwzError::new(value.error);
        if raw.is_success() {
            Ok(ObjectHandle(value.handle))
        } else {
            Err(raw.error())
        }
    }
}

/// Map an object given by ID `id` with the given flags.
pub fn twz_rt_map_object(id: ObjID, flags: MapFlags) -> Result<ObjectHandle> {
    unsafe { nk!(crate::bindings::twz_rt_map_object(id.raw(), flags.bits()).into()) }
}

pub fn twz_rt_get_object_handle(ptr: *const u8) -> Result<ObjectHandle> {
    use crate::error::ObjectError;

    let res = unsafe {
        nk!(crate::bindings::twz_rt_get_object_handle(
            (ptr as *mut u8).cast()
        ))
    };
    if res.id == 0 {
        return Err(TwzError::Object(ObjectError::NotMapped));
    }
    Ok(ObjectHandle(res))
}

pub fn twz_rt_resolve_fot(
    this: &ObjectHandle,
    idx: u64,
    valid_len: usize,
    flags: MapFlags,
) -> Result<ObjectHandle> {
    unsafe {
        nk!(crate::bindings::twz_rt_resolve_fot(
            &this.0 as *const _ as *mut _,
            idx,
            valid_len,
            flags.bits(),
        ))
        .into()
    }
}

impl From<Result<u32>> for crate::bindings::u32_result {
    fn from(value: Result<u32>) -> Self {
        match value {
            Ok(val) => Self {
                val,
                err: RawTwzError::success().raw(),
            },
            Err(e) => Self {
                val: 0,
                err: e.raw(),
            },
        }
    }
}

impl From<crate::bindings::u32_result> for Result<u32> {
    fn from(value: crate::bindings::u32_result) -> Self {
        let raw = RawTwzError::new(value.err);
        if raw.is_success() {
            Ok(value.val)
        } else {
            Err(raw.error())
        }
    }
}

pub fn twz_rt_insert_fot(this: &ObjectHandle, entry: *const u8) -> Result<u32> {
    unsafe {
        let res = nk!(crate::bindings::twz_rt_insert_fot(
            &this.0 as *const _ as *mut _,
            (entry as *mut u8).cast(),
        ));
        res.into()
    }
}

pub fn twz_rt_resolve_fot_local(
    start: *mut u8,
    idx: u64,
    valid_len: usize,
    flags: MapFlags,
) -> *mut u8 {
    unsafe {
        let res = nk!(crate::bindings::twz_rt_resolve_fot_local(
            start.cast(),
            idx,
            valid_len,
            flags.bits()
        ));
        res.cast()
    }
}

use crate::bindings::release_flags;

/// Release a handle. Should be only called by the ObjectHandle drop call.
pub fn twz_rt_release_handle(handle: &mut ObjectHandle, flags: release_flags) {
    unsafe { nk!(crate::bindings::twz_rt_release_handle(&mut handle.0, flags)) }
}

/// Update a handle.
pub fn twz_rt_update_handle(handle: &mut ObjectHandle) -> Result<()> {
    let r = unsafe { nk!(crate::bindings::twz_rt_update_handle(&mut handle.0)) };
    let r = RawTwzError::new(r);
    if r.is_success() {
        Ok(())
    } else {
        Err(r.error())
    }
}

#[deprecated]
pub fn twz_rt_map_two_objects(
    id1: ObjID,
    flags1: MapFlags,
    id2: ObjID,
    flags2: MapFlags,
) -> Result<(ObjectHandle, ObjectHandle)> {
    unsafe {
        let mut res1 = MaybeUninit::uninit();
        let mut res2 = MaybeUninit::uninit();
        nk!(crate::bindings::__twz_rt_map_two_objects(
            id1.raw(),
            flags1.bits(),
            id2.raw(),
            flags2.bits(),
            res1.as_mut_ptr(),
            res2.as_mut_ptr(),
        ));

        let res1 = res1.assume_init();
        let res2 = res2.assume_init();

        let res1: Result<ObjectHandle> = res1.into();
        let res2: Result<ObjectHandle> = res2.into();

        Ok((res1?, res2?))
    }
}

bitflags::bitflags! {
    /// Mapping protections for mapping objects into the address space.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Protections: u16 {
        /// Read allowed.
        const READ = crate::bindings::MAP_FLAG_R as u16;
        /// Write allowed.
        const WRITE = crate::bindings::MAP_FLAG_W as u16;
        /// Exec allowed.
        const EXEC = crate::bindings::MAP_FLAG_X as u16;
    }
}

impl From<Protections> for MapFlags {
    fn from(p: Protections) -> Self {
        let mut f = MapFlags::empty();
        if p.contains(Protections::READ) {
            f.insert(MapFlags::READ);
        }

        if p.contains(Protections::WRITE) {
            f.insert(MapFlags::WRITE);
        }

        if p.contains(Protections::EXEC) {
            f.insert(MapFlags::EXEC);
        }
        f
    }
}

impl From<MapFlags> for Protections {
    fn from(value: MapFlags) -> Self {
        let mut f = Self::empty();
        if value.contains(MapFlags::READ) {
            f.insert(Protections::READ);
        }
        if value.contains(MapFlags::WRITE) {
            f.insert(Protections::WRITE);
        }
        if value.contains(MapFlags::EXEC) {
            f.insert(Protections::EXEC);
        }
        f
    }
}

bitflags::bitflags! {
/// Flags for objects.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Default)]
pub struct MetaFlags: u16 {
    const IMMUTABLE = 1;
}
}

/// A nonce for avoiding object ID collision.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Default)]
#[repr(transparent)]
pub struct Nonce(pub u128);

/// The core metadata that all objects share.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
#[repr(C)]
pub struct MetaInfo {
    /// The ID nonce.
    pub nonce: Nonce,
    /// The object's public key ID.
    pub kuid: ObjID,
    /// The object flags.
    pub flags: MetaFlags,
    /// Default protections
    pub default_prot: Protections,
    /// The number of FOT entries.
    pub fotcount: u16,
    /// The number of meta extensions.
    pub extcount: u16,
}

/// A tag for a meta extension entry.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
#[repr(transparent)]
pub struct MetaExtTag(pub u64);

/// A meta extension entry.
#[repr(C)]
pub struct MetaExt {
    /// The tag.
    pub tag: MetaExtTag,
    /// A tag-specific value.
    pub value: AtomicU64,
}

impl MetaExt {
    pub fn new(tag: MetaExtTag, value: u64) -> Self {
        Self {
            tag,
            value: AtomicU64::new(value),
        }
    }
}

pub const MEXT_EMPTY: MetaExtTag = MetaExtTag(0);
pub const MEXT_SIZED: MetaExtTag = MetaExtTag(1);
/// Modification time in seconds, for objects backed by an external store that records one.
pub const MEXT_MTIME: MetaExtTag = MetaExtTag(2);

/// The maximum size of an object, including null page and meta page(s).
pub const MAX_SIZE: usize = 1024 * 1024 * 1024;
/// The size of the null page.
pub const NULLPAGE_SIZE: usize = 0x1000;

#[repr(C)]
pub struct FotEntry {
    pub values: [u64; 2],
    pub resolver: u64,
    pub flags: AtomicU32,
}

bitflags::bitflags! {
    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
    pub struct FotFlags : u32 {
        const ALLOCATED = 1;
        const ACTIVE = 2;
        const DELETED = 4;
        const RESOLVER = 8;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Default)]
#[repr(C)]
/// Specifications for an object-copy from a source object. The specified ranges are
/// source:[src_start, src_start + len) copied to `<some unspecified destination
/// object>:[dest_start, dest_start + len)`. Each range must start within an object, and end within
/// the object.
pub struct ObjectSource {
    /// The ID of the source object, or zero for filling destination with zero.
    pub id: ObjID,
    /// The offset into the source object to start the copy. If id is zero, this field is reserved
    /// for future use.
    pub src_start: u64,
    /// The offset into the dest object to start the copy or zero.
    pub dest_start: u64,
    /// The length of the copy or zero.
    pub len: usize,
}

impl From<ObjectSource> for crate::bindings::object_source {
    fn from(value: ObjectSource) -> Self {
        Self {
            id: value.id.raw(),
            src_start: value.src_start,
            dest_start: value.dest_start,
            len: value.len as u64,
        }
    }
}

impl ObjectSource {
    /// Construct a new ObjectSource.
    pub fn new_copy(id: ObjID, src_start: u64, dest_start: u64, len: usize) -> Self {
        Self {
            id,
            src_start,
            dest_start,
            len,
        }
    }

    /// Construct a new ObjectSource.
    pub fn new_zero(dest_start: u64, len: usize) -> Self {
        Self {
            id: ObjID::new(0),
            src_start: 0,
            dest_start,
            len,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Default)]
#[repr(u32)]
/// The backing memory type for this object. Currently doesn't do anything.
pub enum BackingType {
    /// The default, let the kernel decide based on the [LifetimeType] of the object.
    #[default]
    Normal = crate::bindings::BACKING_TYPE_NORMAL,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Default)]
#[repr(u32)]
/// The base lifetime type of the object. Note that this does not ensure that the object is stored
/// in a specific type of memory, the kernel is allowed to migrate objects with the Normal
/// [BackingType] as it sees fit. For more information on object lifetime, see [the book](https://twizzler-operating-system.github.io/nightly/book/object_lifetime.html).
pub enum LifetimeType {
    /// This object is volatile, and is expected to be deleted after a power cycle.
    #[default]
    Volatile = crate::bindings::LIFETIME_TYPE_VOLATILE,
    /// This object is persistent, and should be deleted only after an explicit delete call.
    Persistent = crate::bindings::LIFETIME_TYPE_PERSISTENT,
}

impl From<u32> for LifetimeType {
    fn from(value: u32) -> Self {
        match value {
            crate::bindings::LIFETIME_TYPE_VOLATILE => LifetimeType::Volatile,
            crate::bindings::LIFETIME_TYPE_PERSISTENT => LifetimeType::Persistent,
            _ => panic!("Invalid lifetime type"),
        }
    }
}

impl From<u32> for BackingType {
    fn from(value: u32) -> Self {
        match value {
            crate::bindings::BACKING_TYPE_NORMAL => BackingType::Normal,
            _ => panic!("Invalid backing type"),
        }
    }
}

bitflags! {
    /// Flags to pass to the object create system call.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct ObjectCreateFlags: u32 {
        const DELETE = 1;
        const NO_NONCE = 2;
    }
}

bitflags! {
    /// Flags controlling how a particular object tie operates.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct CreateTieFlags: u32 {
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
#[repr(C)]
/// Full object creation specification, minus ties.
pub struct ObjectCreate {
    pub kuid: ObjID,
    pub bt: BackingType,
    pub lt: LifetimeType,
    pub flags: ObjectCreateFlags,
    pub def_prot: Protections,
}
impl ObjectCreate {
    /// Build a new object create specification.
    pub fn new(
        bt: BackingType,
        lt: LifetimeType,
        kuid: Option<ObjID>,
        flags: ObjectCreateFlags,
        def_prot: Protections,
    ) -> Self {
        Self {
            kuid: kuid.unwrap_or_else(|| ObjID::new(0)),
            bt,
            lt,
            flags,
            def_prot,
        }
    }
}

impl Default for ObjectCreate {
    fn default() -> Self {
        Self::new(
            BackingType::Normal,
            LifetimeType::Volatile,
            None,
            ObjectCreateFlags::empty(),
            Protections::all(),
        )
    }
}

impl From<ObjectCreate> for crate::bindings::object_create {
    fn from(value: ObjectCreate) -> Self {
        Self {
            kuid: value.kuid.0,
            lifetime: value.lt as u32,
            backing: value.bt as u32,
            flags: value.flags.bits(),
            prot: value.def_prot.bits() as u32,
        }
    }
}

impl From<object_create> for ObjectCreate {
    fn from(value: object_create) -> Self {
        Self {
            kuid: ObjID::new(value.kuid),
            bt: BackingType::from(value.backing),
            lt: LifetimeType::from(value.lifetime),
            flags: ObjectCreateFlags::from_bits_truncate(value.flags),
            def_prot: Protections::from_bits_truncate(value.prot as u16),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq)]
#[repr(C)]
/// A specification of ties to create.
/// (see [the book](https://twizzler-operating-system.github.io/nightly/book/object_lifetime.html) for more information on ties).
pub struct CreateTieSpec {
    pub id: ObjID,
    pub flags: CreateTieFlags,
}

impl CreateTieSpec {
    /// Create a new CreateTieSpec.
    pub fn new(id: ObjID, flags: CreateTieFlags) -> Self {
        Self { id, flags }
    }
}

impl From<CreateTieSpec> for object_tie {
    fn from(value: CreateTieSpec) -> Self {
        Self {
            id: value.id.raw(),
            flags: value.flags.bits(),
        }
    }
}

unsafe impl Send for sync_info {}
unsafe impl Sync for sync_info {}

impl sync_info {
    pub unsafe fn try_release(&self) -> Result<()> {
        self.release_ptr
            .cast::<AtomicU64>()
            .as_ref()
            .unwrap()
            .compare_exchange(
                self.release_compare,
                self.release_set,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .map_err(|_| TwzError::Resource(ResourceError::Refused))
            .map(|_| ())
    }

    pub unsafe fn set_durable(&self, err: impl Into<RawTwzError>) {
        if self.durable_ptr.is_null() {
            return;
        }

        self.durable_ptr
            .cast::<AtomicU64>()
            .as_ref()
            .unwrap()
            .store(err.into().raw(), Ordering::SeqCst);
    }
}
