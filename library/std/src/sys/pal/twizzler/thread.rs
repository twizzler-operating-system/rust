use crate::ffi::CStr;
use crate::io;
use crate::num::NonZeroUsize;
use crate::time::Duration;

pub struct Thread {
    internal_id: u32,
}

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

pub const DEFAULT_MIN_STACK_SIZE: usize = 1 << 21;

impl Thread {
    // unsafe: see thread::Builder::spawn_unchecked for safety requirements
    pub unsafe fn new(stack: usize, p: Box<dyn FnOnce()>) -> io::Result<Thread> {
        let runtime = twizzler_runtime_api::get_runtime();
        let p = Box::into_raw(Box::new(p));
        let internal_id = runtime.spawn(twizzler_runtime_api::ThreadSpawnArgs {
            stack_size: stack,
            start: thread_start as usize,
            arg: p.expose_provenance(),
        });

        return if let Ok(internal_id) = internal_id {
            Ok(Thread { internal_id })
        } else {
            drop(Box::from_raw(p));
            Err(io::const_io_error!(io::ErrorKind::Uncategorized, &"Unable to create thread!"))
        };

        unsafe extern "C" fn thread_start(main: usize) -> ! {
            {
                Box::from_raw(core::ptr::with_exposed_provenance::<Box<dyn FnOnce()>>(main).cast_mut())();
                // run all destructors
                crate::sys::thread_local::destructors::run();
                crate::rt::thread_cleanup();
            }
            let runtime = twizzler_runtime_api::get_runtime();
            runtime.exit(0);
        }
    }

    pub fn yield_now() {
        let runtime = twizzler_runtime_api::get_runtime();
        runtime.yield_now();
    }

    pub fn set_name(name: &CStr) {
        let runtime = twizzler_runtime_api::get_runtime();
        runtime.set_name(name);
    }

    pub fn sleep(dur: Duration) {
        let runtime = twizzler_runtime_api::get_runtime();
        runtime.sleep(dur);
    }

    pub fn join(self) {
        let runtime = twizzler_runtime_api::get_runtime();
        let _ = runtime.join(self.internal_id, None);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn id(&self) -> u32 {
        self.internal_id
    }
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    let runtime = twizzler_runtime_api::get_runtime();
    Ok(runtime.available_parallelism())
}

