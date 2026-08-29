#![no_std]
#![feature(allocator_api)]
#![allow(internal_features)]
#![feature(rustc_attrs)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![cfg_attr(
    all(feature = "stderr", not(any(feature = "rustc-dep-of-std", doc))),
    feature(io_error_inprogress)
)]
#![cfg_attr(
    all(feature = "stderr", not(any(feature = "rustc-dep-of-std", doc))),
    feature(io_error_more)
)]
#![cfg_attr(feature = "kernel", allow(unused))]

pub mod core;
#[allow(unused_imports)]
pub mod object;

pub mod alloc;
pub mod arch;
pub mod debug;
pub mod exec;
pub mod fd;
pub mod info;
pub mod io;
pub mod marker;
pub mod random;
pub mod thread;
pub mod time;

#[allow(
    non_camel_case_types,
    dead_code,
    non_upper_case_globals,
    improper_ctypes
)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod error;

pub type Result<T> = ::core::result::Result<T, error::TwzError>;

#[macro_export]
macro_rules! nk {
    ($ex:expr) => {{
        #[cfg(feature = "kernel")]
        {
            panic!("tried to call twz_rt from kernel")
        }
        #[allow(unreachable_code)]
        $ex
    }};
}
