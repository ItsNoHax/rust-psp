use psp::fpu::{fcr31, masked, restore_fcr31, set_fcr31, CAUSE, ENABLES};
use psp::test_runner::TestRunner;

pub fn test_main(test_runner: &mut TestRunner) {
    let before = fcr31();
    test_runner.check_true("fpu_masked", masked(before) & (CAUSE | ENABLES) == 0);

    // Round toward zero (RM = 1), every exception masked.
    let written = masked(before) & !3 | 1;
    unsafe { set_fcr31(written) };
    let read = fcr31();
    unsafe { restore_fcr31(before) };
    test_runner.check("fpu_set_fcr31", read & !CAUSE, written);
    test_runner.check("fpu_restore_fcr31", fcr31() & !CAUSE, before & !CAUSE);
}
