use crate::os::raw::c_char;

pub mod args;
pub mod env;
pub mod fd;
pub mod futex;
pub mod os;
pub mod pipe;
pub mod thread;
pub mod time;

pub fn unsupported<T>() -> crate::io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> crate::io::Error {
    crate::io::Error::new(
        crate::io::ErrorKind::Unsupported,
        "operation not supported on Twizzler yet",
    )
}

#[inline]
pub fn abort_internal() -> ! {
    twizzler_rt_abi::core::twz_rt_abort()
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[unsafe(no_mangle)]
// NB. used by both libunwind and libpanic_abort
pub extern "C" fn __rust_abort() {
    abort_internal();
}

// SAFETY: must be called only once during runtime initialization.
// NOTE: this is not guaranteed to run, for example when Rust code is called externally.
pub unsafe fn init(argc: isize, argv: *const *const u8, _sigpipe: u8) {
    args::init(argc, argv);
}

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub unsafe fn cleanup() {}

#[inline]
pub(crate) fn is_interrupted(_errno: i32) -> bool {
    false
}

pub fn decode_error_kind(_errno: i32) -> crate::io::ErrorKind {
    crate::io::ErrorKind::Other
}

#[unsafe(no_mangle)]
#[allow(unreachable_code)]
#[allow(unused_variables)]
pub unsafe extern "C" fn std_entry_from_runtime(
    aux: twizzler_rt_abi::core::BasicAux,
) -> twizzler_rt_abi::core::BasicReturn {
    unsafe extern "C" {
        fn main(argc: isize, argv: *const *const c_char) -> i32;
    }

    crate::sys::os::init_environment(aux.env as *const *const _);
    // If pre_main_hook returns a code, then don't call main and exit with that code instead.
    let code = if let Some(pre_code) = twizzler_rt_abi::core::twz_rt_pre_main_hook() {
        pre_code
    } else {
        main(aux.argc as isize, aux.args as *const *const _)
    };
    twizzler_rt_abi::core::twz_rt_post_main_hook();

    unsafe {
        crate::sys::thread_local::destructors::run();
    }
    crate::rt::thread_cleanup();

    twizzler_rt_abi::core::BasicReturn { code }
}

#[doc(hidden)]
#[allow(dead_code)]
pub trait IsNegative {
    fn is_negative(&self) -> bool;
    fn negate(&self) -> i32;
}

macro_rules! impl_is_negative {
    ($($t:ident)*) => ($(impl IsNegative for $t {
        fn is_negative(&self) -> bool {
            *self < 0
        }

        fn negate(&self) -> i32 {
            i32::try_from(-(*self)).unwrap()
        }
    })*)
}

impl IsNegative for i32 {
    fn is_negative(&self) -> bool {
        *self < 0
    }

    fn negate(&self) -> i32 {
        -(*self)
    }
}
impl_is_negative! { i8 i16 i64 isize }

#[allow(dead_code)]
pub fn cvt<T: IsNegative>(t: T) -> crate::io::Result<T> {
    if t.is_negative() {
        let e = decode_error_kind(t.negate());
        Err(crate::io::Error::from(e))
    } else {
        Ok(t)
    }
}

#[allow(dead_code)]
pub fn cvt_r<T, F>(mut f: F) -> crate::io::Result<T>
where
    T: IsNegative,
    F: FnMut() -> T,
{
    loop {
        match cvt(f()) {
            Err(ref e) if e.is_interrupted() => {}
            other => return other,
        }
    }
}

use twizzler_rt_abi::error::*;

#[stable(feature = "rust1", since = "1.0.0")]
impl From<TwzError> for crate::io::Error {
    fn from(value: TwzError) -> Self {
        let kind: crate::io::ErrorKind = value.into();
        Self::new(kind, value)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl From<TwzError> for crate::io::ErrorKind {
    fn from(value: TwzError) -> Self {
        match value {
            TwzError::Uncategorized(code) => decode_error_kind(code as i32),
            TwzError::Generic(generic_error) => generic_error.into(),
            TwzError::Argument(argument_error) => argument_error.into(),
            TwzError::Resource(resource_error) => resource_error.into(),
            TwzError::Object(object_error) => object_error.into(),
            TwzError::Io(io_error) => io_error.into(),
            TwzError::Naming(naming_error) => naming_error.into(),
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl From<GenericError> for crate::io::ErrorKind {
    fn from(value: GenericError) -> Self {
        match value {
            GenericError::NotSupported => crate::io::ErrorKind::Unsupported,
            GenericError::Internal => crate::io::ErrorKind::Other,
            GenericError::WouldBlock => crate::io::ErrorKind::WouldBlock,
            GenericError::TimedOut => crate::io::ErrorKind::TimedOut,
            GenericError::AccessDenied => crate::io::ErrorKind::PermissionDenied,
            GenericError::NoSuchOperation => crate::io::ErrorKind::Unsupported,
            GenericError::Other => crate::io::ErrorKind::Unsupported,
            GenericError::Interrupted => crate::io::ErrorKind::Interrupted,
            GenericError::InProgress => crate::io::ErrorKind::InProgress,
        }
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl From<ArgumentError> for crate::io::ErrorKind {
    fn from(value: ArgumentError) -> Self {
        match value {
            ArgumentError::InvalidArgument => crate::io::ErrorKind::InvalidInput,
            ArgumentError::WrongType => crate::io::ErrorKind::InvalidInput,
            ArgumentError::InvalidAddress => crate::io::ErrorKind::InvalidInput,
            ArgumentError::BadHandle => crate::io::ErrorKind::InvalidInput,
        }
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl From<ResourceError> for crate::io::ErrorKind {
    fn from(value: ResourceError) -> Self {
        match value {
            ResourceError::OutOfMemory => crate::io::ErrorKind::OutOfMemory,
            ResourceError::OutOfResources => crate::io::ErrorKind::Other,
            ResourceError::OutOfNames => crate::io::ErrorKind::Other,
            ResourceError::Unavailable => crate::io::ErrorKind::ResourceBusy,
            ResourceError::Busy => crate::io::ErrorKind::ResourceBusy,
            ResourceError::NotConnected => crate::io::ErrorKind::NotConnected,
            ResourceError::Unreachable => crate::io::ErrorKind::HostUnreachable,
            ResourceError::Refused => crate::io::ErrorKind::ConnectionRefused,
            ResourceError::NonAtomic => crate::io::ErrorKind::Other,
        }
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl From<ObjectError> for crate::io::ErrorKind {
    fn from(value: ObjectError) -> Self {
        match value {
            ObjectError::MapFailed => crate::io::ErrorKind::Other,
            ObjectError::NotMapped => crate::io::ErrorKind::Other,
            ObjectError::InvalidFote => crate::io::ErrorKind::Other,
            ObjectError::InvalidPtr => crate::io::ErrorKind::Other,
            ObjectError::InvalidMeta => crate::io::ErrorKind::Other,
            ObjectError::BaseTypeMismatch => crate::io::ErrorKind::Other,
            ObjectError::NoSuchObject => crate::io::ErrorKind::NotFound,
        }
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl From<IoError> for crate::io::ErrorKind {
    fn from(value: IoError) -> Self {
        match value {
            IoError::Other => crate::io::ErrorKind::Other,
            IoError::DataLoss => crate::io::ErrorKind::Other,
            IoError::DeviceError => crate::io::ErrorKind::Other,
            IoError::SeekFailed => crate::io::ErrorKind::Other,
            IoError::Reset => crate::io::ErrorKind::ConnectionReset,
        }
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl From<NamingError> for crate::io::ErrorKind {
    fn from(value: NamingError) -> Self {
        match value {
            NamingError::NotFound => crate::io::ErrorKind::NotFound,
            NamingError::AlreadyExists => crate::io::ErrorKind::AlreadyExists,
            NamingError::WrongNameKind => crate::io::ErrorKind::InvalidInput,
            NamingError::AlreadyBound => crate::io::ErrorKind::AddrInUse,
            NamingError::LinkLoop => crate::io::ErrorKind::FilesystemLoop,
            NamingError::NotEmpty => crate::io::ErrorKind::DirectoryNotEmpty,
        }
    }
}
#[stable(feature = "rust1", since = "1.0.0")]
impl From<RawTwzError> for crate::io::ErrorKind {
    fn from(value: RawTwzError) -> Self {
        value.error().into()
    }
}
