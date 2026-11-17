use alloc::alloc::{alloc, dealloc, Layout};
use psp::sys::sceKernelTotalFreeMemSize;
use psp::test_runner::TestRunner;

pub fn test_main(test_runner: &mut TestRunner) {
    test_runner.check_true("alloc_every_alignment", allocates_every_alignment());
    let free = unsafe { sceKernelTotalFreeMemSize() };
    for _ in 0..4 {
        allocates_every_alignment();
    }
    // Each dealloc must free the block its alloc took.
    test_runner.check(
        "alloc_dealloc_frees",
        unsafe { sceKernelTotalFreeMemSize() },
        free,
    );
}

/// Every power-of-two alignment to 4096, at a few sizes: the pointer is
/// aligned and every byte can be written and read back.
fn allocates_every_alignment() -> bool {
    let mut ok = true;
    for shift in 0..=12 {
        for size in [1, 7, 300] {
            let layout = Layout::from_size_align(size, 1 << shift).unwrap();
            unsafe {
                let p = alloc(layout);
                if p.is_null() || p as usize % layout.align() != 0 {
                    return false;
                }
                for i in 0..size {
                    p.add(i).write(i as u8 ^ 0x5a);
                }
                ok &= (0..size).all(|i| p.add(i).read() == i as u8 ^ 0x5a);
                dealloc(p, layout);
            }
        }
    }
    ok
}
