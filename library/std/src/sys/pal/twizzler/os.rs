use crate::error::Error as StdError;
use crate::ffi::{OsStr, OsString};
use crate::marker::PhantomData;
use crate::path::{self, PathBuf};
use crate::{fmt, io, str};

pub fn errno() -> i32 {
    0
}

/// Render an OS code produced by [`RawTwzError::as_os_code`] back into a message.
///
/// `io::Error`'s `Os` repr stores only the code, so this is what its `Display` prints; without it
/// every error carried through `from_raw_os_error` shows up as a bare number.
pub fn error_string(errno: i32) -> String {
    if errno == 0 {
        return "operation successful".to_string();
    }
    twizzler_rt_abi::error::RawTwzError::from_os_code(errno).error().to_string()
}

fn read_name(root: twizzler_rt_abi::fd::NameRoot) -> io::Result<PathBuf> {
    let mut buf = Vec::with_capacity(512);
    loop {
        unsafe {
            let ptr = buf.as_mut_ptr() as *mut u8;
            let slice = core::slice::from_raw_parts_mut(ptr, buf.capacity());
            let res = twizzler_rt_abi::fd::twz_rt_get_nameroot(root, slice)?;
            if res < buf.capacity() {
                buf.set_len(res);
                buf.shrink_to_fit();
                return Ok(PathBuf::from(String::from_utf8(buf).unwrap()));
            }

            // Trigger the internal buffer resizing logic of `Vec` by requiring
            // more space than the current capacity.
            let cap = buf.capacity();
            buf.set_len(cap);
            buf.reserve(1);
        }
    }
}

pub fn getcwd() -> io::Result<PathBuf> {
    read_name(twizzler_rt_abi::fd::NameRoot::Current)
}

pub fn chdir(path: &path::Path) -> io::Result<()> {
    let path = path.to_str().unwrap().as_bytes();
    twizzler_rt_abi::fd::twz_rt_set_nameroot(twizzler_rt_abi::fd::NameRoot::Current, path)?;
    Ok(())
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
    read_name(twizzler_rt_abi::fd::NameRoot::Exe)
}

pub fn temp_dir() -> PathBuf {
    read_name(twizzler_rt_abi::fd::NameRoot::Temp).unwrap_or_else(|_| PathBuf::from("/tmp"))
}

pub fn home_dir() -> Option<PathBuf> {
    read_name(twizzler_rt_abi::fd::NameRoot::Home).ok()
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
