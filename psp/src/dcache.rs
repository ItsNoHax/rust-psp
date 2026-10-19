//! Data-cache maintenance without a system call.
//!
//! `sceKernelDcacheWritebackRange` and friends are system calls; a program
//! that hands the GE or another bus master buffers every frame can write
//! the lines back itself with the Allegrex's `cache` instruction. LLVM's
//! MIPS II assembler rejects the `cache` mnemonic, so each one is emitted
//! as a word with the address in `$a0`.

/// Bytes in a data-cache line: the operations below act on whole lines.
pub const LINE_BYTES: usize = 64;

/// Writes the data-cache lines covering `ptr..ptr + len` back to memory
/// and leaves them in the cache (`cache 0x1a`, Hit Writeback), for a
/// buffer the CPU reads again. Returns once the writes have reached memory.
///
/// # Safety
///
/// `ptr..ptr + len` must be mapped.
pub unsafe fn writeback_range(ptr: *const u8, len: usize) {
    let mut line = ptr as usize & !(LINE_BYTES - 1);
    let end = ptr as usize + len;
    while line < end {
        core::arch::asm!(".word 0xbc9a0000", in("$4") line, options(nostack, preserves_flags));
        line += LINE_BYTES;
    }
    core::arch::asm!("sync", options(nostack, preserves_flags));
}

/// Writes the data-cache lines covering `ptr..ptr + len` back to memory
/// and invalidates them (`cache 0x1b`, Hit Writeback Invalidate), which
/// frees them for other data. Returns once the writes have reached memory.
///
/// # Safety
///
/// `ptr..ptr + len` must be mapped.
pub unsafe fn writeback_invalidate_range(ptr: *const u8, len: usize) {
    let mut line = ptr as usize & !(LINE_BYTES - 1);
    let end = ptr as usize + len;
    while line < end {
        core::arch::asm!(".word 0xbc9b0000", in("$4") line, options(nostack, preserves_flags));
        line += LINE_BYTES;
    }
    core::arch::asm!("sync", options(nostack, preserves_flags));
}
