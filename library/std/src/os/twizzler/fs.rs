#![stable(feature = "metadata_ext", since = "1.1.0")]

use crate::fs::Metadata;
use crate::sys_common::AsInner;

/// OS-specific extensions to [`fs::Metadata`].
///
/// [`fs::Metadata`]: crate::fs::Metadata
#[stable(feature = "metadata_ext", since = "1.1.0")]
pub trait MetadataExt {
    #[stable(feature = "metadata_ext2", since = "1.8.0")]
    fn st_size(&self) -> u64;
    #[stable(feature = "metadata_ext2", since = "1.8.0")]
    fn st_objid(&self) -> u128;
    #[stable(feature = "metadata_ext2", since = "1.8.0")]
    fn st_mode(&self) -> u32;
    #[stable(feature = "metadata_ext2", since = "1.8.0")]
    fn st_atime(&self) -> i64;
    #[stable(feature = "metadata_ext2", since = "1.8.0")]
    fn st_atime_nsec(&self) -> i64;
    #[stable(feature = "metadata_ext2", since = "1.8.0")]
    fn st_mtime(&self) -> i64;
    #[stable(feature = "metadata_ext2", since = "1.8.0")]
    fn st_mtime_nsec(&self) -> i64;
    #[stable(feature = "metadata_ext2", since = "1.8.0")]
    fn st_ctime(&self) -> i64;
    #[stable(feature = "metadata_ext2", since = "1.8.0")]
    fn st_ctime_nsec(&self) -> i64;
}

#[stable(feature = "metadata_ext", since = "1.1.0")]
impl MetadataExt for Metadata {
    fn st_size(&self) -> u64 {
        self.as_inner().size()
    }

    fn st_objid(&self) -> u128 {
        self.as_inner().objid().raw()
    }

    fn st_mode(&self) -> u32 {
        self.as_inner().mode()
    }

    fn st_atime(&self) -> i64 {
        self.as_inner().atime().as_secs() as i64
    }

    fn st_atime_nsec(&self) -> i64 {
        self.as_inner().atime().as_nanos() as i64
    }

    fn st_mtime(&self) -> i64 {
        self.as_inner().mtime().as_secs() as i64
    }

    fn st_mtime_nsec(&self) -> i64 {
        self.as_inner().mtime().as_nanos() as i64
    }

    fn st_ctime(&self) -> i64 {
        self.as_inner().ctime().as_secs() as i64
    }

    fn st_ctime_nsec(&self) -> i64 {
        self.as_inner().ctime().as_nanos() as i64
    }
}
