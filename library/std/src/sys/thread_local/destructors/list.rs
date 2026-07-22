use crate::alloc::System;
use crate::cell::RefCell;
use crate::sys::thread_local::guard;

#[thread_local]
static DTORS: RefCell<Vec<(*mut u8, unsafe extern "C" fn(*mut u8)), System>> =
    RefCell::new(Vec::new_in(System));

pub unsafe fn register(t: *mut u8, dtor: unsafe extern "C" fn(*mut u8)) {
    let Ok(mut dtors) = DTORS.try_borrow_mut() else {
        rtabort!("the System allocator may not use TLS with destructors")
    };
    guard::enable();
    dtors.push((t, dtor));
}

/// The [`guard`] module contains platform-specific functions which will run this
/// function on thread exit if [`guard::enable`] has been called.
///
/// # Safety
///
/// May only be run on thread exit to guarantee that there are no live references
/// to TLS variables while they are destroyed.
pub unsafe fn run() {
    loop {
        let mut dtors = DTORS.borrow_mut();
        match dtors.pop() {
            Some((t, dtor)) => {
                drop(dtors);
                unsafe {
                    dtor(t);
                }
            }
            None => {
                // Free the list memory.
                *dtors = Vec::new_in(System);
                break;
            }
        }
    }
}

/// Run destructors for a foreign thread's TLS block.
/// `tp` is the thread pointer (FS base on x86_64) of the exiting thread.
#[cfg(target_os = "twizzler")]
pub unsafe fn run_for_tp(my_tp: *mut u8, tp: *mut u8) {
    unsafe {
        let dtors_addr = crate::ptr::addr_of!(DTORS) as *mut u8;
        let offset = dtors_addr.offset_from(my_tp);
        // Apply that same offset to the foreign thread's TP
        let foreign_dtors = tp.offset(offset)
            as *mut RefCell<Vec<(*mut u8, unsafe extern "C" fn(*mut u8)), System>>;
        loop {
            let mut dtors = (*foreign_dtors).borrow_mut();
            match dtors.pop() {
                Some((t, dtor)) => {
                    drop(dtors);
                    dtor(t);
                }
                None => {
                    *dtors = Vec::new_in(System);
                    break;
                }
            }
        }
    }
}
