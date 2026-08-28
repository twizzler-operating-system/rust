#![stable(feature = "rust1", since = "1.0.0")]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod ffi;
pub mod fs;

// Now that this target claims the unix family, `os::unix::process` exists and is this very
// file. Including it again here compiles a second copy, and its impls of shared traits
// (`AsRawFd`/`AsFd`/`IntoRawFd`/`From<OwnedFd>`) for shared types collide with the first.
#[stable(feature = "rust1", since = "1.0.0")]
pub use crate::os::unix::process;

/// A prelude for conveniently writing platform-specific code.
///
/// Includes all extension traits, and some important type definitions.
#[stable(feature = "rust1", since = "1.0.0")]
pub mod prelude {
    #[doc(no_inline)]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub use super::ffi::{OsStrExt, OsStringExt};
    #[doc(no_inline)]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub use super::process::{ChildExt, ExitStatusExt};
    #[doc(no_inline)]
    #[stable(feature = "rust1", since = "1.0.0")]
    pub use crate::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
}
