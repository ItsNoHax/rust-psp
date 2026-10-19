//! The FPU control and status register, FCR31.
//!
//! A PSP thread starts with FCR31 `0x00000E00`: invalid operation,
//! divide-by-zero and overflow trap. Rust and LLVM assume the IEEE default,
//! every exception masked; a program that wants that default writes it with
//! [`set_fcr31`]`(`[`masked`]`(`[`fcr31`]`()))` on entry to each thread.

/// The cause bits (`E V Z O U I`, bits 12-17). A write that sets a cause
/// bit together with its enable traps at the `ctc1` itself.
pub const CAUSE: u32 = 0x3F << 12;
/// The enable bits (`V Z O U I`, bits 7-11): a set bit makes that exception
/// trap.
pub const ENABLES: u32 = 0x1F << 7;
/// The sticky flag bits (`V Z O U I`, bits 2-6).
pub const FLAGS: u32 = 0x1F << 2;

/// `fcr31` with every exception masked and no cause pending.
pub const fn masked(fcr31: u32) -> u32 {
    fcr31 & !(CAUSE | ENABLES)
}

/// Reads this thread's FCR31.
pub fn fcr31() -> u32 {
    let v: u32;
    // SAFETY: `cfc1` of FCR31 only reads this thread's FPU control word.
    // A fixed register: the allocator may otherwise pick `$at`, which the
    // assembler reserves.
    unsafe { core::arch::asm!("cfc1 $8, $31", out("$8") v, options(nomem, nostack)) };
    v
}

/// Writes this thread's FCR31.
///
/// # Safety
///
/// Code compiled around the write assumes the floating-point environment it
/// was compiled for. A value with a cause bit and its enable both set traps
/// at the write; [`restore_fcr31`] puts back a value read earlier.
pub unsafe fn set_fcr31(v: u32) {
    core::arch::asm!("ctc1 $8, $31", in("$8") v, options(nomem, nostack));
}

/// Puts back `saved`, an FCR31 read earlier, with no cause bit pending.
///
/// Operations since the read can leave cause bits set (a NaN compare's
/// invalid), and writing the enables back while one is set traps. This
/// writes `saved` with its causes and enables cleared, runs an exact
/// `add.s` that clears the cause bits, then writes `saved` itself; one
/// `asm!` block, so nothing can be scheduled between the three.
///
/// # Safety
///
/// As for [`set_fcr31`].
pub unsafe fn restore_fcr31(saved: u32) {
    core::arch::asm!(
        "ctc1 {quiet}, $31",
        "add.s {r}, {one}, {one}",
        "ctc1 {saved}, $31",
        quiet = in(reg) masked(saved),
        saved = in(reg) saved,
        one = in(freg) 1.0f32,
        r = out(freg) _,
        options(nomem, nostack),
    );
}
