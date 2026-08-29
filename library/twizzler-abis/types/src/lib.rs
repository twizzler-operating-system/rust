#![no_std]

pub type ObjID = bindings::objid;

#[allow(
    non_camel_case_types,
    dead_code,
    non_upper_case_globals,
    improper_ctypes
)]
pub(crate) mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
