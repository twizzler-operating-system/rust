use crate::error::Error as StdError;
use crate::ffi::{OsStr, OsString};
use crate::marker::PhantomData;
use crate::path::{self, PathBuf};
use crate::sys::unsupported;
use crate::{fmt, io, str};

pub fn errno() -> i32 {
    0
}

pub fn error_string(_errno: i32) -> String {
    "operation successful".to_string()
}

pub fn getcwd() -> io::Result<PathBuf> {
    unsupported()
}

pub fn chdir(_: &path::Path) -> io::Result<()> {
    unsupported()
}

pub struct SplitPaths<'a>(!, PhantomData<&'a ()>);

pub fn split_paths(_unparsed: &OsStr) -> SplitPaths<'_> {
    panic!("unsupported")
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        self.0
    }
}

#[derive(Debug)]
pub struct JoinPathsError;

pub fn join_paths<I, T>(_paths: I) -> Result<OsString, JoinPathsError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    Err(JoinPathsError)
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "not supported on twizzler yet".fmt(f)
    }
}

impl StdError for JoinPathsError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "not supported on twizzler yet"
    }
}

pub fn current_exe() -> io::Result<PathBuf> {
    unsupported()
}

pub fn temp_dir() -> PathBuf {
    PathBuf::from("/tmp")
}

pub fn home_dir() -> Option<PathBuf> {
    None
}

pub fn exit(code: i32) -> ! {
    twizzler_rt_abi::core::twz_rt_exit(code)
}

pub fn getpid() -> u32 {
    unimplemented!()
}

pub fn getppid() -> u32 {
    unimplemented!()
}
