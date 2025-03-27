#![allow(dead_code)]

use core::ffi::CStr;

use twizzler_rt_abi::fd::{FdInfo, FdKind, NameEntry};
use twizzler_rt_abi::object::ObjID;

use crate::ffi::OsString;
use crate::io::{self, BorrowedCursor, Error, ErrorKind, IoSlice, IoSliceMut, SeekFrom};
use crate::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, RawFd};
use crate::path::{Path, PathBuf};
use crate::sys::common::small_c_string::run_path_with_cstr;
use crate::sys::fd::FileDesc;
use crate::sys::pal::twizzler::time;
use crate::sys::time::SystemTime;
use crate::sys::unsupported;
pub use crate::sys_common::fs::{copy, exists};
use crate::sys_common::{AsInner, AsInnerMut, FromInner, IntoInner};
use crate::time::Duration;

#[derive(Debug)]
pub struct File(FileDesc);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FileAttr {
    sz: u64,
    ty: FileType,
    perms: FilePermissions,
    times: FileTimes,
    id: ObjID,
    mode: u32,
}

#[derive(Debug)]
pub struct ReadDir {
    file: FileDesc,
    pos: usize,
    buf: [NameEntry; 128],
    bufpos: usize,
    buflen: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DirEntry {
    name: String,
    meta: FileAttr,
}

impl From<NameEntry> for DirEntry {
    fn from(value: NameEntry) -> Self {
        Self {
            name: String::from_utf8_lossy(value.name_bytes()).into_owned(),
            meta: FileAttr::from(FdInfo::from(value.info)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct OpenOptions {
    // generic
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct FileTimes {
    created: SystemTime,
    modified: SystemTime,
    accessed: SystemTime,
}

impl Default for FileTimes {
    fn default() -> Self {
        Self { created: time::UNIX_EPOCH, modified: time::UNIX_EPOCH, accessed: time::UNIX_EPOCH }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct FilePermissions(u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum FileType {
    Regular,
    Directory,
    SymLink,
}

#[derive(Debug)]
pub struct DirBuilder {}

impl FileAttr {
    pub fn size(&self) -> u64 {
        self.sz
    }

    pub fn objid(&self) -> ObjID {
        self.id
    }

    pub fn perm(&self) -> FilePermissions {
        self.perms
    }

    pub fn file_type(&self) -> FileType {
        self.ty
    }

    pub fn modified(&self) -> io::Result<SystemTime> {
        Ok(self.times.modified)
    }

    pub fn accessed(&self) -> io::Result<SystemTime> {
        Ok(self.times.accessed)
    }

    pub fn created(&self) -> io::Result<SystemTime> {
        Ok(self.times.created)
    }

    pub fn atime(&self) -> Duration {
        self.times.accessed.0
    }

    pub fn mtime(&self) -> Duration {
        self.times.modified.0
    }

    pub fn ctime(&self) -> Duration {
        self.times.created.0
    }

    pub fn mode(&self) -> u32 {
        self.mode
    }
}

impl From<FdInfo> for FileAttr {
    fn from(value: FdInfo) -> Self {
        Self {
            sz: value.size,
            ty: match value.kind {
                FdKind::Regular => FileType::Regular,
                FdKind::Directory => FileType::Directory,
                FdKind::SymLink => FileType::SymLink,
                _ => FileType::Regular, //TODO
            },
            perms: FilePermissions(value.unix_mode),
            times: FileTimes {
                created: time::SystemTime(value.created),
                accessed: time::SystemTime(value.accessed),
                modified: time::SystemTime(value.modified),
            },
            id: value.id.into(),
            mode: value.unix_mode,
        }
    }
}

impl FilePermissions {
    pub fn readonly(&self) -> bool {
        // TODO
        false
    }

    pub fn set_readonly(&mut self, _readonly: bool) {}
}

impl FileTimes {
    pub fn set_accessed(&mut self, _t: SystemTime) {}
    pub fn set_modified(&mut self, _t: SystemTime) {}
}

impl FileType {
    pub fn is_dir(&self) -> bool {
        matches!(self, FileType::Directory)
    }

    pub fn is_file(&self) -> bool {
        matches!(self, FileType::Regular)
    }

    pub fn is_symlink(&self) -> bool {
        matches!(self, FileType::SymLink)
    }
}

impl ReadDir {
    fn new(file: FileDesc) -> Self {
        Self { file, pos: 0, bufpos: 0, buf: [NameEntry::default(); 128], buflen: 0 }
    }

    fn read_next(&mut self) -> bool {
        if let Some(count) = twizzler_rt_abi::fd::twz_rt_fd_enumerate_names(
            self.file.as_raw_fd(),
            &mut self.buf,
            self.pos,
        ) {
            if count == 0 {
                return false;
            }
            self.bufpos = 0;
            self.buflen = count;
            self.pos += count;
            true
        } else {
            false
        }
    }
}

impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<io::Result<DirEntry>> {
        if self.bufpos < self.buflen {
            let de = DirEntry::from(self.buf[self.bufpos]);
            self.bufpos += 1;
            return Some(Ok(de));
        }
        if self.read_next() {
            return self.next();
        }
        None
    }
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.name.clone().into()
    }

    pub fn file_name(&self) -> OsString {
        self.name.clone().into()
    }

    pub fn metadata(&self) -> io::Result<FileAttr> {
        Ok(self.meta)
    }

    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(self.metadata()?.ty)
    }
}

impl OpenOptions {
    pub fn new() -> OpenOptions {
        OpenOptions {
            // generic
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    pub fn append(&mut self, append: bool) {
        self.append = append;
        self.write = true;
    }
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new;
    }
}

impl File {
    pub fn open(path: &Path, opts: &OpenOptions) -> io::Result<File> {
        run_path_with_cstr(path, &|path| File::open_c(&path, opts))
    }

    pub fn open_c(path: &CStr, opts: &OpenOptions) -> io::Result<File> {
        let kind = if opts.create_new {
            twizzler_rt_abi::bindings::CREATE_KIND_NEW
        } else if opts.create {
            twizzler_rt_abi::bindings::CREATE_KIND_EITHER
        } else {
            twizzler_rt_abi::bindings::CREATE_KIND_EXISTING
        };
        let create = twizzler_rt_abi::bindings::create_options { kind };
        let mut flags = 0;
        if opts.read {
            flags |= twizzler_rt_abi::bindings::OPEN_FLAG_READ;
        }
        if opts.write {
            flags |= twizzler_rt_abi::bindings::OPEN_FLAG_WRITE;
        }
        if opts.append {
            flags |= twizzler_rt_abi::bindings::OPEN_FLAG_TAIL;
        }
        if opts.truncate {
            flags |= twizzler_rt_abi::bindings::OPEN_FLAG_TRUNCATE;
        }
        let fd = twizzler_rt_abi::fd::twz_rt_fd_copen(path, create, flags)?;
        Ok(File(unsafe { FileDesc::from_raw_fd(fd) }))
    }

    pub fn file_attr(&self) -> io::Result<FileAttr> {
        let info = twizzler_rt_abi::fd::twz_rt_fd_get_info(self.as_raw_fd())
            .ok_or(ErrorKind::Unsupported)?;
        Ok(info.into())
    }

    pub fn fsync(&self) -> io::Result<()> {
        twizzler_rt_abi::fd::twz_rt_fd_sync(self.as_raw_fd());
        Ok(())
    }

    pub fn datasync(&self) -> io::Result<()> {
        self.fsync()
    }

    pub fn truncate(&self, size: u64) -> io::Result<()> {
        twizzler_rt_abi::fd::twz_rt_fd_truncate(self.as_raw_fd(), size)
            .map_err(|_| ErrorKind::Other)?;
        Ok(())
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        // TODO: use twizzler vec io
        crate::io::default_read_vectored(|buf| self.read(buf), bufs)
    }

    pub fn is_read_vectored(&self) -> bool {
        // TODO: use twizzler vec io
        false
    }

    pub fn read_buf(&self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        crate::io::default_read_buf(|buf| self.read(buf), cursor)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        // TODO: use twizzler vec io
        crate::io::default_write_vectored(|buf| self.write(buf), bufs)
    }

    pub fn is_write_vectored(&self) -> bool {
        // TODO: use twizzler vec io
        false
    }

    pub fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    pub fn seek(&self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }

    pub fn duplicate(&self) -> io::Result<File> {
        let fd = twizzler_rt_abi::fd::twz_rt_fd_dup(self.as_raw_fd())?;
        Ok(File(unsafe { FileDesc::from_raw_fd(fd) }))
    }

    pub fn set_permissions(&self, _perm: FilePermissions) -> io::Result<()> {
        Err(Error::from_raw_os_error(22))
    }

    pub fn set_times(&self, _times: FileTimes) -> io::Result<()> {
        Err(Error::from_raw_os_error(22))
    }
}

impl DirBuilder {
    pub fn new() -> DirBuilder {
        DirBuilder {}
    }

    pub fn mkdir(&self, p: &Path) -> io::Result<()> {
        twizzler_rt_abi::fd::twz_rt_fd_mkns(
            p.as_os_str().to_str().ok_or(ErrorKind::InvalidFilename)?,
        )?;
        Ok(())
    }
}

pub fn readdir(p: &Path) -> io::Result<ReadDir> {
    let mut open = OpenOptions::new();
    open.read(true);
    let file = File::open(p, &open)?;
    Ok(ReadDir::new(file.0))
}

pub fn unlink(p: &Path) -> io::Result<()> {
    twizzler_rt_abi::fd::twz_rt_fd_remove(
        p.as_os_str().to_str().ok_or(ErrorKind::InvalidFilename)?,
    )?;
    Ok(())
}

pub fn rename(_old: &Path, _new: &Path) -> io::Result<()> {
    // TODO
    unsupported()
}

pub fn set_perm(_p: &Path, _perm: FilePermissions) -> io::Result<()> {
    unsupported()
}

pub fn rmdir(p: &Path) -> io::Result<()> {
    unlink(p)
}

pub fn remove_dir_all(_path: &Path) -> io::Result<()> {
    unsupported()
}

pub fn try_exists(path: &Path) -> io::Result<bool> {
    stat(path)?;
    Ok(true)
}

pub fn readlink(p: &Path) -> io::Result<PathBuf> {
    let mut buf = [0; 4096];
    let len = twizzler_rt_abi::fd::twz_rt_fd_readlink(
        p.as_os_str().to_str().ok_or(ErrorKind::InvalidFilename)?,
        &mut buf,
    )?;
    let s = crate::str::from_utf8(&buf[..len]).map_err(|_| ErrorKind::InvalidFilename)?;
    Ok(PathBuf::from(s))
}

pub fn symlink(original: &Path, link: &Path) -> io::Result<()> {
    twizzler_rt_abi::fd::twz_rt_fd_symlink(
        link.as_os_str().to_str().ok_or(ErrorKind::InvalidFilename)?,
        original.as_os_str().to_str().ok_or(ErrorKind::InvalidFilename)?,
    )?;
    Ok(())
}

pub fn link(_src: &Path, _dst: &Path) -> io::Result<()> {
    unsupported()
}

pub fn stat(p: &Path) -> io::Result<FileAttr> {
    let file = File::open(p, &OpenOptions::new())?;
    file.file_attr()
}

pub fn lstat(p: &Path) -> io::Result<FileAttr> {
    stat(p)
}

pub fn canonicalize(p: &Path) -> io::Result<PathBuf> {
    Ok(PathBuf::from(p))
}

impl AsInner<FileDesc> for File {
    #[inline]
    fn as_inner(&self) -> &FileDesc {
        &self.0
    }
}

impl AsInnerMut<FileDesc> for File {
    #[inline]
    fn as_inner_mut(&mut self) -> &mut FileDesc {
        &mut self.0
    }
}

impl IntoInner<FileDesc> for File {
    fn into_inner(self) -> FileDesc {
        self.0
    }
}

impl FromInner<FileDesc> for File {
    fn from_inner(file_desc: FileDesc) -> Self {
        Self(file_desc)
    }
}

impl AsFd for File {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for File {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for File {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for File {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(FromRawFd::from_raw_fd(raw_fd))
    }
}

use twizzler_rt_abi::fd::OpenError;
#[stable(feature = "twizzler_io", since = "1.0")]
impl From<OpenError> for io::Error {
    fn from(value: OpenError) -> Self {
        let kind = match value {
            OpenError::Other => io::ErrorKind::Other,
            OpenError::LookupFail => io::ErrorKind::NotFound,
            OpenError::PermissionDenied => io::ErrorKind::PermissionDenied,
            OpenError::InvalidArgument => io::ErrorKind::InvalidInput,
        };

        io::Error::new(kind, Box::new(value))
    }
}
