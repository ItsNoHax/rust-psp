extern "C" {
    /// Call a function accepting 5 32-bit integer arguments via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i5(
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        ptr: extern "C" fn(u32, u32, u32, u32, u32) -> u32,
    ) -> u32;

    /// Call a function accepting 6 32-bit integer arguments via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i6(
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        ptr: extern "C" fn(u32, u32, u32, u32, u32, u32) -> u32,
    ) -> u32;

    /// Call a function accepting 7 32-bit integer arguments via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i7(
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
        ptr: extern "C" fn(u32, u32, u32, u32, u32, u32, u32) -> u32,
    ) -> u32;

    /// Call a function accepting 8 32-bit integer arguments via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i8(
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
        h: u32,
        ptr: extern "C" fn(u32, u32, u32, u32, u32, u32, u32, u32) -> u32,
    ) -> u32;

    /// Call a function accepting 9 32-bit integer arguments via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i9(
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
        h: u32,
        i: u32,
        ptr: extern "C" fn(u32, u32, u32, u32, u32, u32, u32, u32, u32) -> u32,
    ) -> u32;

    /// Call a function with the signature `fn(i32, i64, i32) -> i64` via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i_ii_i_rii(a: u32, b: u64, c: u32, ptr: extern "C" fn(u32, u64, u32) -> u64) -> u64;

    /// Call a function with the signature `fn(i32, i64, i32) -> i32` via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i_ii_i_ri(a: u32, b: u64, c: u32, ptr: extern "C" fn(u32, u64, u32) -> u32) -> u32;

    /// Call a function with the signature `fn(i32, i64, i32, i32) -> i32` via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i_ii_i_i_ri(
        a: u32,
        b: u64,
        c: u32,
        d: u32,
        ptr: extern "C" fn(u32, u64, u32, u32) -> u32,
    ) -> u32;

    /// Call a function with the signature `fn(i32, i32, i32, i64) -> i32` via the MIPS-EABI ABI.
    ///
    /// This is not safe to call with a function that expects any other ABI.
    pub fn i_i_i_ii_ri(
        a: u32,
        b: u32,
        c: u32,
        d: u64,
        ptr: extern "C" fn(u32, u32, u32, u64) -> u32,
    ) -> u32;
}

/// Converts an argument to a mapper's register parameter: same-size values
/// are transmuted, and `u8`/`u16`-sized ones are zero-extended to 32 bits, as
/// the caller would extend them.
#[doc(hidden)]
#[inline(always)]
pub unsafe fn reg<T, R>(v: T) -> R {
    use core::mem::{size_of, transmute_copy, ManuallyDrop};
    const {
        assert!(
            size_of::<T>() == size_of::<R>() || (size_of::<T>() < 4 && size_of::<R>() == 4),
            "argument does not fit the EABI mapper's register",
        )
    };
    let v = ManuallyDrop::new(v);
    if size_of::<T>() == size_of::<R>() {
        transmute_copy(&*v)
    } else {
        let mut r = 0u32;
        core::ptr::copy_nonoverlapping(
            &*v as *const T as *const u8,
            &mut r as *mut u32 as *mut u8,
            size_of::<T>(),
        );
        transmute_copy(&r)
    }
}

// Potential resource:
// (scroll down to table) https://www.linux-mips.org/wiki/P32_Linux_ABI
//
// Page 3-18: http://web.archive.org/web/20040930224745/http://www.caldera.com/developers/devspecs/mipsabi.pdf
// Copied from PDF:
// Despite the fact that some or all of the arguments to a function are passed
// in registers, always allocate space on the stack for all arguments. This
// stack space should be a structure large enough to contain all the arguments,
// aligned according to normal structure rules (after promotion and structure
// return pointer insertion). The locations within the stack frame used for
// arguments are called the home locations.
#[cfg(target_os = "psp")]
core::arch::global_asm!(
    r#"
        .section .text
        .global i5
        i5:
            // Store the return register as we are calling a function manually.
            addiu $sp, -32
            sw $ra, 8($sp)

            // Load argument 5 into register t0. In MIPS-EABI, t0 is actually a4.
            lw $t0, 48($sp)

            // Load and call the bridged function.
            lw $t1, 52($sp)
            jalr $t1

            // Restore the stack and return.
            lw $ra, 8($sp)
            addiu $sp, 32
            jr $ra

        // Argument slots 4 and 5 are the same for these signatures: in both
        // ABIs a 64-bit argument starts on an even slot.
        .global i6
        .global i_ii_i_i_ri
        .global i_i_i_ii_ri
        i6:
        i_ii_i_i_ri:
        i_i_i_ii_ri:
            addiu $sp, -32
            sw $ra, 8($sp)

            lw $t0, 48($sp)
            lw $t1, 52($sp)

            lw $t2, 56($sp)
            jalr $t2

            lw $ra, 8($sp)
            addiu $sp, 32
            jr $ra

        .global i7
        i7:
            addiu $sp, -32
            sw $ra, 8($sp)

            lw $t0, 48($sp)
            lw $t1, 52($sp)
            lw $t2, 56($sp)

            lw $t3, 60($sp)
            jalr $t3

            lw $ra, 8($sp)
            addiu $sp, 32
            jr $ra

        .global i8
        i8:
            addiu $sp, -32
            sw $ra, 8($sp)

            lw $t0, 48($sp)
            lw $t1, 52($sp)
            lw $t2, 56($sp)
            lw $t3, 60($sp)

            lw $t4, 64($sp)
            jalr $t4

            lw $ra, 8($sp)
            addiu $sp, 32
            jr $ra

        .global i9
        i9:
            addiu $sp, -32
            sw $ra, 8($sp)

            lw $t0, 48($sp)
            lw $t1, 52($sp)
            lw $t2, 56($sp)
            lw $t3, 60($sp)

            // MIPS-EABI has 8 argument registers. The 9th argument goes on
            // the stack, where the callee reads it at 0($sp).
            lw $t4, 64($sp)
            sw $t4, 0($sp)

            lw $t4, 68($sp)
            jalr $t4

            lw $ra, 8($sp)
            addiu $sp, 32
            jr $ra

        .global i_ii_i_rii
        .global i_ii_i_ri
        i_ii_i_rii:
        i_ii_i_ri:
            addiu $sp, -32
            sw $ra, 8($sp)

            lw $t0, 48($sp)
            lw $t1, 52($sp)
            jalr $t1

            lw $ra, 8($sp)
            addiu $sp, 32
            jr $ra
    "#
);
