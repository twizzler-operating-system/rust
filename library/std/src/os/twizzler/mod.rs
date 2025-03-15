#![stable(feature = "rust1", since = "1.0.0")]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod ffi;
pub mod fs;

/// A prelude for conveniently writing platform-specific code.
///
/// Includes all extension traits, and some important type definitions.
#[stable(feature = "rust1", since = "1.0.0")]
pub mod prelude {
    #[doc(no_inline)]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub use super::ffi::{OsStrExt, OsStringExt};
}

#[stable(feature = "io_safety", since = "1.63.0")]
impl From<crate::os::fd::OwnedFd> for crate::process::Stdio {
    #[inline]
    fn from(_fd: crate::os::fd::OwnedFd) -> crate::process::Stdio {
        // TODO
        Self::inherit()
    }
}
