//! Twizzler thread stacks are runtime allocations, not OS-visible mappings, so the bounds come
//! from the runtime. Without this backend `guess_os_stack_limit` was `None` on every thread, and
//! `maybe_grow` therefore grew (allocated, zeroed and unmapped a segment) on every outermost
//! call -- in rustc, once per executed query.
//!
//! `{0, 0}` from the runtime means "unknown" (e.g. a compartment's main thread, whose stack the
//! monitor built): report `None` so `maybe_grow` stays on its conservative grow-always path
//! there. Never map unknown to `Some(0)` -- a zero limit reads as unbounded remaining stack, and
//! these stacks are guard-page-less heap allocations, so "never grow" means a silent overflow.
//!
//! The failure modes are asymmetric, which is what makes the zero-guard the load-bearing line:
//! a limit that is too HIGH (e.g. returning `start + len`) makes `remaining_stack`'s
//! `saturating_sub` clamp toward zero, which reads as "no space" and merely grows every time --
//! today's behavior, harmless. A limit that is too LOW (zero, or any bogus small value) reads as
//! unbounded remaining and never grows -- silent overflow. Only the low direction is dangerous;
//! do not "simplify" the guard.

#[repr(C)]
struct StackBounds {
    start: usize,
    len: usize,
}

extern "C-unwind" {
    fn twz_rt_get_stack_bounds() -> StackBounds;
}

pub unsafe fn guess_os_stack_limit() -> Option<usize> {
    let bounds = twz_rt_get_stack_bounds();
    if bounds.start == 0 || bounds.len == 0 {
        return None;
    }
    Some(bounds.start)
}
