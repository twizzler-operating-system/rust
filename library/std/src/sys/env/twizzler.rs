use core::slice::memchr;

pub use super::common::Env;
use crate::collections::HashMap;
use crate::ffi::{CStr, OsStr, OsString, c_char};
use crate::io;
use crate::os::twizzler::ffi::OsStringExt;
use crate::sync::Mutex;

static ENV: Mutex<Option<HashMap<OsString, OsString>>> = Mutex::new(None);

// mlibc's environment vector. There is exactly one per process (libc.so is loaded once), while a
// process can hold several copies of this crate: a program that links std statically -- rustc,
// via librustc_driver.so -- still runs alongside the runtime's libstd.so. Only the copy whose
// `std_entry_from_runtime` the dynamic linker bound gets `init_environment` called on it, so every
// other copy has to fill `ENV` itself or it sees no environment at all.
unsafe extern "C" {
    static environ: *const *const c_char;
}

fn parse_env(env: *const *const c_char) -> HashMap<OsString, OsString> {
    let mut map = HashMap::new();
    if env.is_null() {
        return map;
    }

    unsafe {
        let mut cur = env;
        while !(*cur).is_null() {
            if let Some((key, value)) = parse(CStr::from_ptr(*cur).to_bytes()) {
                map.insert(key, value);
            }
            cur = cur.add(1);
        }
    }

    return map;

    fn parse(input: &[u8]) -> Option<(OsString, OsString)> {
        // Strategy (copied from glibc): Variable name and value are separated
        // by an ASCII equals sign '='. Since a variable name must not be
        // empty, allow variable names starting with an equals sign. Skip all
        // malformed lines.
        if input.is_empty() {
            return None;
        }
        let pos = memchr::memchr(b'=', &input[1..]).map(|p| p + 1);
        pos.map(|p| {
            (
                OsStringExt::from_vec(input[..p].to_vec()),
                OsStringExt::from_vec(input[p + 1..].to_vec()),
            )
        })
    }
}

fn with_env<T>(f: impl FnOnce(&mut HashMap<OsString, OsString>) -> T) -> T {
    let mut guard = ENV.lock().unwrap();
    let map = guard.get_or_insert_with(|| parse_env(unsafe { environ }));
    f(map)
}

pub fn init_environment(env: *const *const c_char) {
    *ENV.lock().unwrap() = Some(parse_env(env));
}

/// Returns a vector of (variable, value) byte-vector pairs for all the
/// environment variables of the current process.
pub fn env() -> Env {
    Env::new(with_env(|env| env.iter().map(|(key, value)| (key.clone(), value.clone())).collect()))
}

pub fn getenv(k: &OsStr) -> Option<OsString> {
    with_env(|env| env.get(k).cloned())
}

pub unsafe fn setenv(k: &OsStr, v: &OsStr) -> io::Result<()> {
    let (k, v) = (k.to_owned(), v.to_owned());
    with_env(|env| env.insert(k, v));
    Ok(())
}

pub unsafe fn unsetenv(k: &OsStr) -> io::Result<()> {
    with_env(|env| env.remove(k));
    Ok(())
}
