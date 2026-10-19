use psp::test_runner::TestRunner;

// A library rust-psp does not bind, imported from outside the crate.
// PPSSPP and ARK's SystemControl both export it.
psp::psp_extern! {
    #![name = "KUBridge"]
    #![flags = 0x4001]
    #![version = (0x00, 0x00)]

    #[psp(0x24331850)]
    /// The PSP model: 0 for a PSP-1000, 1 for a PSP-2000, ...
    pub fn kuKernelGetModel() -> i32;
}

pub fn test_main(test_runner: &mut TestRunner) {
    let model = unsafe { kuKernelGetModel() };
    test_runner.dbg(
        "psp_extern",
        &alloc::format!("kuKernelGetModel() = {}", model),
    );
    test_runner.check_true("psp_extern_kubridge_model", (0..=10).contains(&model));
}
