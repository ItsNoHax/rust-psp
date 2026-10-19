use alloc::vec::Vec;
use psp::test_runner::TestRunner;

extern "C" {
    fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn memset(dst: *mut u8, value: i32, n: usize) -> *mut u8;
}

// The functions under test back `vec!`, `fill` and `copy_from_slice` too,
// so every buffer is built and checked byte by byte from these formulas.

fn src_byte(i: usize) -> u8 {
    (i * 7 + 3) as u8
}

fn dst_byte(i: usize) -> u8 {
    (i * 13 + 5) as u8
}

fn buffer(len: usize, byte: fn(usize) -> u8) -> Vec<u8> {
    (0..len).map(byte).collect()
}

pub fn test_main(test_runner: &mut TestRunner) {
    test_runner.check_true("memcpy", copies(|d, s, n| unsafe { memcpy(d, s, n) }));
    test_runner.check_true(
        "memmove_disjoint",
        copies(|d, s, n| unsafe { memmove(d, s, n) }),
    );
    test_runner.check_true("memmove_overlapping", moves_overlapping_ranges());
    test_runner.check_true("memset", fills_every_length_and_alignment());
}

/// Every length to 69 at every source and destination alignment.
fn copies(copy: impl Fn(*mut u8, *const u8, usize) -> *mut u8) -> bool {
    for len in 0..70 {
        for so in 0..4 {
            for do_ in 0..4 {
                let src = buffer(len + 8, src_byte);
                let mut dst = buffer(len + 8, dst_byte);
                let ret = copy(
                    unsafe { dst.as_mut_ptr().add(do_) },
                    unsafe { src.as_ptr().add(so) },
                    len,
                );
                if ret != unsafe { dst.as_mut_ptr().add(do_) } {
                    return false;
                }
                for (i, &b) in dst.iter().enumerate() {
                    let want = if (do_..do_ + len).contains(&i) {
                        src_byte(i - do_ + so)
                    } else {
                        dst_byte(i)
                    };
                    if b != want {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn moves_overlapping_ranges() -> bool {
    for len in 0..70 {
        for from in 0..8 {
            for to in 0..8 {
                let mut buf = buffer(len + 8, src_byte);
                let p = buf.as_mut_ptr();
                unsafe { memmove(p.add(to), p.add(from), len) };
                for (i, &b) in buf.iter().enumerate() {
                    let want = if (to..to + len).contains(&i) {
                        src_byte(i - to + from)
                    } else {
                        src_byte(i)
                    };
                    if b != want {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn fills_every_length_and_alignment() -> bool {
    for len in 0..70 {
        for off in 0..4 {
            let mut buf = buffer(len + 8, dst_byte);
            unsafe { memset(buf.as_mut_ptr().add(off), 0xA5, len) };
            for (i, &b) in buf.iter().enumerate() {
                let want = if (off..off + len).contains(&i) {
                    0xA5
                } else {
                    dst_byte(i)
                };
                if b != want {
                    return false;
                }
            }
        }
    }
    true
}
