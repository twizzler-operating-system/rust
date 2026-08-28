use crate::error::Error as StdError;
use crate::ffi::{OsStr, OsString};
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

/// Twizzler paths are `/`-separated and, everywhere else in this PAL, required to be UTF-8, so
/// `PATH` splits on `:` exactly as it does on unix. This used to `panic!("unsupported")`, which
/// took out any caller that merely *reads* PATH -- rustc's `get_linker` does, to hand a PATH down
/// to the linker it spawns, so linking on-target aborted before the linker was ever invoked.
pub struct SplitPaths<'a> {
    rest: Option<&'a str>,
}

pub const PATH_SEPARATOR: char = ':';

pub fn split_paths(unparsed: &OsStr) -> SplitPaths<'_> {
    // A non-UTF-8 PATH yields no entries rather than an error: this signature cannot report one,
    // and every other name-taking call in this PAL rejects non-UTF-8 outright.
    SplitPaths { rest: unparsed.to_str() }
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;
    fn next(&mut self) -> Option<PathBuf> {
        // Matches unix: "" yields one empty path, and a trailing separator yields a final empty
        // path. Callers rely on the count, so empties are kept rather than skipped.
        let rest = self.rest?;
        Some(match rest.find(PATH_SEPARATOR) {
            Some(i) => {
                self.rest = Some(&rest[i + PATH_SEPARATOR.len_utf8()..]);
                PathBuf::from(&rest[..i])
            }
            None => {
                self.rest = None;
                PathBuf::from(rest)
            }
        })
    }
}

#[derive(Debug)]
pub struct JoinPathsError;

/// Inverse of [`split_paths`]. Fixed alongside it rather than separately: `get_linker` calls
/// `join_paths(..).unwrap()` two lines after `split_paths`, so an unconditional `Err` here just
/// moves the abort rather than removing it.
pub fn join_paths<I, T>(paths: I) -> Result<OsString, JoinPathsError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    let mut joined = OsString::new();
    for (i, path) in paths.enumerate() {
        let path = path.as_ref();
        // A segment containing the separator would not survive a round trip through split_paths,
        // so refuse it rather than silently producing a different list.
        let Some(s) = path.to_str() else { return Err(JoinPathsError) };
        if s.contains(PATH_SEPARATOR) {
            return Err(JoinPathsError);
        }
        if i > 0 {
            joined.push(":");
        }
        joined.push(path);
    }
    Ok(joined)
}

impl fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "path segment contains separator `:` or is not valid UTF-8".fmt(f)
    }
}

impl StdError for JoinPathsError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        "failed to join paths"
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
    // Twizzler has no process ids; hand out a stable per-instance value (std::process::id is
    // used for lock/scratch file naming) until the runtime exposes a real instance id.
    use crate::sync::OnceLock;
    static PSEUDO_PID: OnceLock<u32> = OnceLock::new();
    *PSEUDO_PID.get_or_init(|| {
        let mut bytes = [0u8; 4];
        crate::sys::random::fill_bytes(&mut bytes);
        u32::from_ne_bytes(bytes) | 1
    })
}

pub fn getppid() -> u32 {
    unimplemented!()
}
