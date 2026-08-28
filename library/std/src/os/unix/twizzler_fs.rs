//! Unix-specific extensions to primitives in the [`std::fs`] module, implemented over the
//! Twizzler runtime ABI rather than libc.
//!
//! Twizzler claims `target_family = "unix"` so the ecosystem's `cfg(unix)` code is reachable, but
//! std stays on `twz_rt_*` — see `sys/fs/twizzler.rs`. These are the same traits a conventional
//! unix exposes, with bodies backed by the runtime.
//!
//! Where Twizzler has no analogue for a `struct stat` field, this reports a fixed value rather
//! than a fabricated one: `dev`, `uid`, `gid` and `rdev` are 0 and `nlink` is 1. `ino` is the low
//! 64 bits of the object ID, which is the closest thing to a stable per-object identity.
//!
//! [`std::fs`]: crate::fs

#![stable(feature = "rust1", since = "1.0.0")]

use twizzler_rt_abi::io::{IoCtx, IoFlags};

use crate::fs::{self, Metadata, OpenOptions, Permissions};
use crate::io;
use crate::os::fd::AsRawFd;
use crate::path::Path;
use crate::sys::AsInner;
use crate::sys::fs::FilePermissions;
use crate::sys::FromInner;

/// Unix-specific extensions to [`fs::File`].
#[stable(feature = "file_offset", since = "1.15.0")]
pub trait FileExt {
    /// Reads a number of bytes starting from a given offset.
    #[stable(feature = "file_offset", since = "1.15.0")]
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize>;

    /// Reads the exact number of bytes required to fill `buf` from a given offset.
    #[stable(feature = "rw_exact_all_at", since = "1.33.0")]
    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            match self.read_at(buf, offset) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &mut buf[n..];
                    offset += n as u64;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err(io::Error::READ_EXACT_EOF)
        } else {
            Ok(())
        }
    }

    /// Writes a number of bytes starting from a given offset.
    #[stable(feature = "file_offset", since = "1.15.0")]
    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize>;

    /// Attempts to write an entire buffer starting from a given offset.
    #[stable(feature = "rw_exact_all_at", since = "1.33.0")]
    fn write_all_at(&self, mut buf: &[u8], mut offset: u64) -> io::Result<()> {
        while !buf.is_empty() {
            match self.write_at(buf, offset) {
                Ok(0) => {
                    return Err(io::Error::WRITE_ALL_EOF);
                }
                Ok(n) => {
                    buf = &buf[n..];
                    offset += n as u64
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

#[stable(feature = "file_offset", since = "1.15.0")]
impl FileExt for fs::File {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        let mut ctx = IoCtx::new(Some(offset), IoFlags::empty(), None);
        Ok(twizzler_rt_abi::io::twz_rt_fd_pread(self.as_raw_fd(), buf, &mut ctx)?)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        let mut ctx = IoCtx::new(Some(offset), IoFlags::empty(), None);
        Ok(twizzler_rt_abi::io::twz_rt_fd_pwrite(self.as_raw_fd(), buf, &mut ctx)?)
    }
}

/// Unix-specific extensions to [`fs::Permissions`].
#[stable(feature = "fs_ext", since = "1.1.0")]
pub trait PermissionsExt {
    /// Returns the underlying raw `st_mode` bits.
    #[stable(feature = "fs_ext", since = "1.1.0")]
    fn mode(&self) -> u32;

    /// Sets the underlying raw bits.
    #[stable(feature = "fs_ext", since = "1.1.0")]
    fn set_mode(&mut self, mode: u32);

    /// Creates a new instance from the given mode bits.
    #[stable(feature = "fs_ext", since = "1.1.0")]
    fn from_mode(mode: u32) -> Self;
}

#[stable(feature = "fs_ext", since = "1.1.0")]
impl PermissionsExt for Permissions {
    fn mode(&self) -> u32 {
        self.as_inner().mode()
    }

    // Recorded, not enforced: `sys::fs::twizzler`'s `set_perm` is a no-op, so a mode set here is
    // observable through `mode()` and nowhere else. That matches the platform's existing
    // behaviour rather than pretending permission changes take effect.
    fn set_mode(&mut self, mode: u32) {
        *self = Permissions::from_inner(FilePermissions::from_mode(mode));
    }

    fn from_mode(mode: u32) -> Permissions {
        Permissions::from_inner(FilePermissions::from_mode(mode))
    }
}

/// Unix-specific extensions to [`fs::OpenOptions`].
#[stable(feature = "fs_ext", since = "1.1.0")]
pub trait OpenOptionsExt {
    /// Sets the mode bits that a new file will be created with.
    #[stable(feature = "fs_ext", since = "1.1.0")]
    fn mode(&mut self, mode: u32) -> &mut Self;

    /// Passes custom flags to the `flags` argument of `open`.
    #[stable(feature = "open_options_ext", since = "1.10.0")]
    fn custom_flags(&mut self, flags: i32) -> &mut Self;
}

#[stable(feature = "fs_ext", since = "1.1.0")]
impl OpenOptionsExt for OpenOptions {
    // Accepted and ignored. The runtime's `twz_rt_fd_open` takes no creation mode and no raw
    // flag word, and silently dropping the value is what a caller like `OpenOptions::mode(0o600)`
    // already gets today. Returning an error instead would break callers that set a mode purely
    // out of unix habit.
    fn mode(&mut self, _mode: u32) -> &mut OpenOptions {
        self
    }

    fn custom_flags(&mut self, _flags: i32) -> &mut OpenOptions {
        self
    }
}

/// Unix-specific extensions to [`fs::Metadata`].
#[stable(feature = "metadata_ext", since = "1.1.0")]
pub trait MetadataExt {
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn dev(&self) -> u64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn ino(&self) -> u64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn mode(&self) -> u32;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn nlink(&self) -> u64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn uid(&self) -> u32;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn gid(&self) -> u32;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn rdev(&self) -> u64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn size(&self) -> u64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn atime(&self) -> i64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn atime_nsec(&self) -> i64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn mtime(&self) -> i64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn mtime_nsec(&self) -> i64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn ctime(&self) -> i64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn ctime_nsec(&self) -> i64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn blksize(&self) -> u64;
    #[stable(feature = "metadata_ext", since = "1.1.0")]
    fn blocks(&self) -> u64;
}

#[stable(feature = "metadata_ext", since = "1.1.0")]
impl MetadataExt for Metadata {
    fn dev(&self) -> u64 {
        0
    }

    // Low 64 bits of the object ID. Callers use `ino` for identity (walkdir's symlink-loop
    // detection, hard-link dedup), so a per-object value beats 0; the truncation is a real, if
    // remote, collision risk that a 64-bit field leaves no way to avoid.
    fn ino(&self) -> u64 {
        self.as_inner().objid().raw() as u64
    }

    fn mode(&self) -> u32 {
        self.as_inner().mode()
    }

    fn nlink(&self) -> u64 {
        1
    }

    fn uid(&self) -> u32 {
        0
    }

    fn gid(&self) -> u32 {
        0
    }

    fn rdev(&self) -> u64 {
        0
    }

    fn size(&self) -> u64 {
        self.as_inner().size()
    }

    fn atime(&self) -> i64 {
        self.as_inner().atime().as_secs() as i64
    }

    fn atime_nsec(&self) -> i64 {
        self.as_inner().atime().subsec_nanos() as i64
    }

    fn mtime(&self) -> i64 {
        self.as_inner().mtime().as_secs() as i64
    }

    fn mtime_nsec(&self) -> i64 {
        self.as_inner().mtime().subsec_nanos() as i64
    }

    fn ctime(&self) -> i64 {
        self.as_inner().ctime().as_secs() as i64
    }

    fn ctime_nsec(&self) -> i64 {
        self.as_inner().ctime().subsec_nanos() as i64
    }

    fn blksize(&self) -> u64 {
        4096
    }

    fn blocks(&self) -> u64 {
        self.as_inner().size().div_ceil(512)
    }
}

/// Unix-specific extensions for [`fs::FileType`].
#[stable(feature = "file_type_ext", since = "1.5.0")]
pub trait FileTypeExt {
    #[stable(feature = "file_type_ext", since = "1.5.0")]
    fn is_block_device(&self) -> bool;
    #[stable(feature = "file_type_ext", since = "1.5.0")]
    fn is_char_device(&self) -> bool;
    #[stable(feature = "file_type_ext", since = "1.5.0")]
    fn is_fifo(&self) -> bool;
    #[stable(feature = "file_type_ext", since = "1.5.0")]
    fn is_socket(&self) -> bool;
}

// `sys::fs::twizzler::FileType` is Regular/Directory/SymLink only: the runtime's other fd kinds
// (pty, socket, compartment, kconsole) do not survive into a `FileType`, so none of these can be
// true today. If that enum grows, this is the place that must grow with it.
#[stable(feature = "file_type_ext", since = "1.5.0")]
impl FileTypeExt for fs::FileType {
    fn is_block_device(&self) -> bool {
        false
    }

    fn is_char_device(&self) -> bool {
        false
    }

    fn is_fifo(&self) -> bool {
        false
    }

    fn is_socket(&self) -> bool {
        false
    }
}

/// Unix-specific extension methods for [`fs::DirEntry`].
#[stable(feature = "dir_entry_ext", since = "1.1.0")]
pub trait DirEntryExt {
    #[stable(feature = "dir_entry_ext", since = "1.1.0")]
    fn ino(&self) -> u64;
}

#[stable(feature = "dir_entry_ext", since = "1.1.0")]
impl DirEntryExt for fs::DirEntry {
    // Not free, unlike the unix `d_ino`: directory enumeration fills `fd_info` with zeros apart
    // from name and kind (see `sys::fs::twizzler::DirEntry::metadata`), so the cached attr would
    // report 0 for every entry — identical inodes for distinct files, which is worse than a stat.
    fn ino(&self) -> u64 {
        self.as_inner().metadata().map(|m| m.objid().raw() as u64).unwrap_or(0)
    }
}

/// Creates a new symbolic link on the filesystem.
#[stable(feature = "symlink", since = "1.1.0")]
pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) -> io::Result<()> {
    crate::sys::fs::symlink(original.as_ref(), link.as_ref())
}

/// Unix-specific extensions to [`fs::DirBuilder`].
#[stable(feature = "dir_builder", since = "1.6.0")]
pub trait DirBuilderExt {
    /// Sets the mode to create new directories with.
    #[stable(feature = "dir_builder", since = "1.6.0")]
    fn mode(&mut self, mode: u32) -> &mut Self;
}

#[stable(feature = "dir_builder", since = "1.6.0")]
impl DirBuilderExt for fs::DirBuilder {
    // Ignored, for the same reason as `OpenOptionsExt::mode`.
    fn mode(&mut self, _mode: u32) -> &mut fs::DirBuilder {
        self
    }
}
