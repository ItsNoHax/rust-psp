//! The module's start arguments, as `module_start` received them.
//!
//! The system starts a module with a block of NUL-terminated strings; for an
//! EBOOT the first is the path it was started from (`argv[0]`, e.g.
//! `ms0:/PSP/GAME/APP/EBOOT.PBP`). [`module!`](crate::module) copies the
//! block here on the main thread before `psp_main` runs, so [`args`] works on
//! any thread for the life of the program.

use core::sync::atomic::{AtomicUsize, Ordering};

/// Bytes of the argument block kept. An argument that does not fit whole is
/// dropped, with every one after it.
pub const CAPACITY: usize = 512;

/// Not yet set by [`init`].
const UNSET: usize = usize::MAX;

static mut BLOCK: [u8; CAPACITY] = [0; CAPACITY];
static LEN: AtomicUsize = AtomicUsize::new(UNSET);

/// Copies the argument block: `len` bytes at `argp`. Called by
/// [`module!`](crate::module) on the main thread before `psp_main`; later
/// calls do nothing.
///
/// # Safety
///
/// `argp` must be valid for `len` bytes, or `len` must be 0.
#[doc(hidden)]
pub unsafe fn init(len: usize, argp: *const u8) {
    if LEN.load(Ordering::Acquire) != UNSET {
        return;
    }
    let src: &[u8] = if len == 0 || argp.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(argp, len)
    };
    let kept = if src.len() <= CAPACITY {
        src.len()
    } else {
        // Up to and including the last NUL that fits.
        src[..CAPACITY]
            .iter()
            .rposition(|&b| b == 0)
            .map_or(0, |i| i + 1)
    };
    // SAFETY: written once, before `LEN` publishes it; only read after.
    core::ptr::copy_nonoverlapping(
        src.as_ptr(),
        core::ptr::addr_of_mut!(BLOCK).cast::<u8>(),
        kept,
    );
    LEN.store(kept, Ordering::Release);
}

/// The kept argument block, empty if the module was started without one.
fn block() -> &'static [u8] {
    match LEN.load(Ordering::Acquire) {
        UNSET => &[],
        // SAFETY: `BLOCK[..n]` is never written again once `LEN` is set.
        n => unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(BLOCK).cast::<u8>(), n) },
    }
}

/// The start arguments, each without its NUL.
pub fn args() -> Args {
    Args { rest: block() }
}

/// The path the module was started from (`argv[0]`), if it was given one.
pub fn arg0() -> Option<&'static [u8]> {
    args().next()
}

/// Iterator over the start arguments ([`args`]).
#[derive(Clone, Debug)]
pub struct Args {
    rest: &'static [u8],
}

impl Iterator for Args {
    type Item = &'static [u8];

    fn next(&mut self) -> Option<&'static [u8]> {
        if self.rest.is_empty() {
            return None;
        }
        let (arg, rest) = match self.rest.iter().position(|&b| b == 0) {
            Some(n) => (&self.rest[..n], &self.rest[n + 1..]),
            None => (self.rest, &[][..]),
        };
        self.rest = rest;
        Some(arg)
    }
}
