use core::mem::{size_of, MaybeUninit};
use psp::sys::{self, SceKernelThreadInfo, SceUid};
use psp::test_runner::TestRunner;

/// `main.rs` passes this to `psp::module!`.
pub const MAIN_STACK_BYTES: i32 = 320 * 1024;

pub fn test_main(test_runner: &mut TestRunner) {
    let stack_size = unsafe {
        let mut info = MaybeUninit::<SceKernelThreadInfo>::zeroed();
        (*info.as_mut_ptr()).size = size_of::<SceKernelThreadInfo>();
        let thid = SceUid(sys::sceKernelGetThreadId());
        if sys::sceKernelReferThreadStatus(thid, info.as_mut_ptr()) < 0 {
            -1
        } else {
            info.assume_init().stack_size
        }
    };
    test_runner.check("main_thread_stack_size", stack_size, MAIN_STACK_BYTES);
}
