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
    fn st_objid(&self) -> twizzler_rt_abi::object::ObjID;
}

#[stable(feature = "metadata_ext", since = "1.1.0")]
impl MetadataExt for Metadata {
    fn st_size(&self) -> u64 {
        self.as_inner().size()
    }

    fn st_objid(&self) -> twizzler_rt_abi::object::ObjID {
        self.as_inner().objid()
    }
}
