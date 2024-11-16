use crate::ffi::CStr;
use crate::io;
use crate::num::NonZeroUsize;
use crate::time::Duration;

use twizzler_rt_abi::{thread::{ThreadId}};

pub struct Thread {
    internal_id: ThreadId,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

pub const DEFAULT_MIN_STACK_SIZE: usize = 1 << 21;

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        let p = Box::into_raw(Box::new(p));
        let internal_id = twizzler_rt_abi::thread::twz_rt_spawn_thread(twizzler_rt_abi::thread::ThreadSpawnArgs {
            stack_size: stack,
            start: thread_start as usize,
            arg: p.expose_provenance(),
        });

        return if let Ok(internal_id) = internal_id {
            Ok(Thread { internal_id })
        } else {
            drop(Box::from_raw(p));
            Err(io::const_io_error!(io::ErrorKind::Uncategorized, &"unable to create thread"))
        };

        unsafe extern "C" fn thread_start(main: usize) -> ! {
            {
                Box::from_raw(core::ptr::with_exposed_provenance::<Box<dyn FnOnce()>>(main).cast_mut())();
                // run all destructors
                crate::sys::thread_local::destructors::run();
                crate::rt::thread_cleanup();
            }
            twizzler_rt_abi::core::twz_rt_exit(0);
        }
    }

    pub fn yield_now() {
        twizzler_rt_abi::thread::twz_rt_yield();
    }

    pub fn set_name(name: &CStr) {
        twizzler_rt_abi::thread::twz_rt_set_thread_name(name);
    }

    pub fn sleep(dur: Duration) {
        twizzler_rt_abi::thread::twz_rt_sleep(dur);
    }

    pub fn join(self) {
        // If join returns an error, then we'll just assume that thread has joined. Since this function
        // doesn't timeout, that error case can't happen either.
        let _ = twizzler_rt_abi::thread::twz_rt_join_thread(self.internal_id, None);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn id(&self) -> ThreadId {
        self.internal_id
    }
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    let info = twizzler_rt_abi::info::twz_rt_get_sysinfo();
    Ok(info.available_parallelism.try_into().unwrap_or(NonZeroUsize::new(1).unwrap()))
}

