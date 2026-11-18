use psp::test_runner::TestRunner;

pub fn test_main(test_runner: &mut TestRunner) {
    let arg0 = psp::args::arg0();
    test_runner.check_true("args_arg0_given", arg0.is_some());
    let arg0 = arg0.unwrap_or(&[]);
    // A device path: `ms0:/...`, `umd0:/...`, `host0:/...`.
    let colon = arg0.iter().position(|&b| b == b':');
    test_runner.check_true(
        "args_arg0_is_a_path",
        colon.is_some_and(|i| i > 0 && arg0.get(i + 1) == Some(&b'/')),
    );
    test_runner.check_true(
        "args_arg0_printable",
        arg0.iter().all(|b| (b' '..=b'~').contains(b)),
    );
    // Same answer every call.
    test_runner.check_true("args_stable", psp::args::arg0() == Some(arg0));
    test_runner.check_true("args_count", psp::args::args().count() >= 1);
}
