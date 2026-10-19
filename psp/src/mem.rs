//! Word-at-a-time bodies for the C `memcpy`, `memmove` and `memset` that
//! `alloc_impl` exports.
//!
//! Every large Rust move or zeroing calls these, so a byte per loop
//! iteration (five instructions a byte on the Allegrex) costs real frame
//! time. These copy four bytes per load and store whenever source and
//! destination share their alignment, unrolled four words deep.
//!
//! Every access is volatile so the compiler cannot recognise a loop here as
//! a copy and call `memcpy` from inside `memcpy`.

use core::ptr::{read_volatile, write_volatile};

/// Copies `n` bytes from `src` to `dst`, front to back.
///
/// # Safety
/// Valid ranges; when they overlap `dst` must not be above `src`.
pub unsafe fn copy_forward(dst: *mut u8, src: *const u8, n: usize) {
    let (mut d, mut s, mut n) = (dst, src, n);
    if n >= 8 && (d as usize ^ s as usize) & 3 == 0 {
        while d as usize & 3 != 0 {
            write_volatile(d, read_volatile(s));
            d = d.add(1);
            s = s.add(1);
            n -= 1;
        }
        let (mut dw, mut sw) = (d as *mut u32, s as *const u32);
        while n >= 16 {
            let (a, b, c, e) = (
                read_volatile(sw),
                read_volatile(sw.add(1)),
                read_volatile(sw.add(2)),
                read_volatile(sw.add(3)),
            );
            write_volatile(dw, a);
            write_volatile(dw.add(1), b);
            write_volatile(dw.add(2), c);
            write_volatile(dw.add(3), e);
            dw = dw.add(4);
            sw = sw.add(4);
            n -= 16;
        }
        while n >= 4 {
            write_volatile(dw, read_volatile(sw));
            dw = dw.add(1);
            sw = sw.add(1);
            n -= 4;
        }
        d = dw as *mut u8;
        s = sw as *const u8;
    }
    while n > 0 {
        write_volatile(d, read_volatile(s));
        d = d.add(1);
        s = s.add(1);
        n -= 1;
    }
}

/// Copies `n` bytes from `src` to `dst`, back to front.
///
/// # Safety
/// Valid ranges; when they overlap `dst` must not be below `src`.
pub unsafe fn copy_backward(dst: *mut u8, src: *const u8, n: usize) {
    let (mut d, mut s, mut n) = (dst.add(n), src.add(n), n);
    if n >= 8 && (d as usize ^ s as usize) & 3 == 0 {
        while d as usize & 3 != 0 {
            d = d.sub(1);
            s = s.sub(1);
            write_volatile(d, read_volatile(s));
            n -= 1;
        }
        let (mut dw, mut sw) = (d as *mut u32, s as *const u32);
        while n >= 4 {
            dw = dw.sub(1);
            sw = sw.sub(1);
            write_volatile(dw, read_volatile(sw));
            n -= 4;
        }
        d = dw as *mut u8;
        s = sw as *const u8;
    }
    while n > 0 {
        d = d.sub(1);
        s = s.sub(1);
        write_volatile(d, read_volatile(s));
        n -= 1;
    }
}

/// Copies `n` bytes between possibly overlapping ranges.
///
/// # Safety
/// Both ranges valid for `n` bytes.
pub unsafe fn move_bytes(dst: *mut u8, src: *const u8, n: usize) {
    if (dst as usize) <= (src as usize) {
        copy_forward(dst, src, n);
    } else {
        copy_backward(dst, src, n);
    }
}

/// Fills `n` bytes at `dst` with `value`.
///
/// # Safety
/// `dst` valid for `n` bytes.
pub unsafe fn fill(dst: *mut u8, value: u8, n: usize) {
    let (mut d, mut n) = (dst, n);
    if n >= 8 {
        while d as usize & 3 != 0 {
            write_volatile(d, value);
            d = d.add(1);
            n -= 1;
        }
        let word = u32::from(value) * 0x0101_0101;
        let mut dw = d as *mut u32;
        while n >= 16 {
            write_volatile(dw, word);
            write_volatile(dw.add(1), word);
            write_volatile(dw.add(2), word);
            write_volatile(dw.add(3), word);
            dw = dw.add(4);
            n -= 16;
        }
        while n >= 4 {
            write_volatile(dw, word);
            dw = dw.add(1);
            n -= 4;
        }
        d = dw as *mut u8;
    }
    while n > 0 {
        write_volatile(d, value);
        d = d.add(1);
        n -= 1;
    }
}
