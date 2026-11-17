//! Imports with more than four argument registers go through an EABI mapper,
//! which passes arguments 5 to 8 in `$t0`-`$t3` and the 9th on the stack.
//! Without one, the callee reads garbage for them.

use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};
use psp::sys::{self, EventFlagAttributes, EventFlagWaitTypes, SceKernelSysClock, SceUid};
use psp::test_runner::TestRunner;

const USER_PARTITION: i32 = 2;
const ERROR_WAIT_TIMEOUT: i32 = 0x8002_01a8_u32 as i32;

pub fn test_main(test_runner: &mut TestRunner) {
    unsafe {
        let sema = sys::sceKernelCreateSema(b"abi_sema\0".as_ptr(), 0, 0, 1, null_mut());
        test_runner.check_true("abi_create_sema", sema.0 > 0);
        sys::sceKernelDeleteSema(sema);

        let pipe = sys::sceKernelCreateMsgPipe(
            b"abi_pipe\0".as_ptr(),
            USER_PARTITION,
            0,
            256 as *mut c_void,
            null_mut(),
        );
        test_runner.check_true("abi_create_msg_pipe", pipe.0 > 0);
        sys::sceKernelDeleteMsgPipe(pipe);

        let vpl =
            sys::sceKernelCreateVpl(b"abi_vpl\0".as_ptr(), USER_PARTITION, 0, 4096, null_mut());
        test_runner.check_true("abi_create_vpl", vpl.0 > 0);
        sys::sceKernelDeleteVpl(vpl);

        let fpl =
            sys::sceKernelCreateFpl(b"abi_fpl\0".as_ptr(), USER_PARTITION, 0, 64, 4, null_mut());
        test_runner.check_true("abi_create_fpl", fpl > 0);
        sys::sceKernelDeleteFpl(SceUid(fpl));

        // The timeout is the 5th argument.
        let flag = sys::sceKernelCreateEventFlag(
            b"abi_flag\0".as_ptr(),
            EventFlagAttributes::empty(),
            0,
            null_mut(),
        );
        let mut timeout = 1000u32;
        let r =
            sys::sceKernelWaitEventFlag(flag, 1, EventFlagWaitTypes::AND, null_mut(), &mut timeout);
        test_runner.check("abi_wait_event_flag_timeout", r, ERROR_WAIT_TIMEOUT);
        sys::sceKernelDeleteEventFlag(flag);
    }

    vtimer_handler_wide(test_runner);
    trampolines(test_runner);
}

unsafe extern "C" fn vtimer_handler(_: SceUid, _: i64, _: i64, _: *mut c_void) -> u32 {
    0
}

/// A 64-bit argument in `$a2:$a3`, then two more in `$t0`/`$t1`.
fn vtimer_handler_wide(test_runner: &mut TestRunner) {
    const TIME: i64 = 0x1_0000_0123;
    let common = 0x1234 as *mut c_void;
    unsafe {
        let timer = sys::sceKernelCreateVTimer(b"abi_vtimer\0".as_ptr(), null_mut());
        let r = sys::sceKernelSetVTimerHandlerWide(timer, TIME, vtimer_handler, common);
        test_runner.check("abi_set_vtimer_handler_wide", r, 0);

        // Not `zeroed()`: `handler` is a non-nullable fn pointer until the
        // kernel fills it in.
        let mut info = core::mem::MaybeUninit::<sys::SceKernelVTimerInfo>::zeroed();
        addr_of_mut!((*info.as_mut_ptr()).size)
            .write(core::mem::size_of::<sys::SceKernelVTimerInfo>());
        sys::sceKernelReferVTimerStatus(timer, info.as_mut_ptr());
        let info = info.assume_init();
        let SceKernelSysClock { low, hi } = info.schedule;
        test_runner.check("abi_vtimer_schedule", (hi as i64) << 32 | low as i64, TIME);
        test_runner.check(
            "abi_vtimer_handler",
            info.handler as usize,
            vtimer_handler as usize,
        );
        test_runner.check("abi_vtimer_common", info.common, common);
        sys::sceKernelDeleteVTimer(timer);
    }
}

/// Where `abi_test_eabi_probe` found its arguments: `$a0`-`$a3`, `$t0`-`$t3`
/// and `0($sp)`, as an EABI callee reads arguments 1 to 9.
static mut SEEN: [u32; 9] = [0; 9];

core::arch::global_asm!(
    ".global abi_test_eabi_probe",
    "abi_test_eabi_probe:",
    "lui $v1, %hi({seen})",
    "addiu $v1, $v1, %lo({seen})",
    "sw $a0, 0($v1)",
    "sw $a1, 4($v1)",
    "sw $a2, 8($v1)",
    "sw $a3, 12($v1)",
    "sw $t0, 16($v1)",
    "sw $t1, 20($v1)",
    "sw $t2, 24($v1)",
    "sw $t3, 28($v1)",
    "lw $v0, 0($sp)",
    "sw $v0, 32($v1)",
    "li $v0, 0x600d",
    "jr $ra",
    seen = sym SEEN,
);

extern "C" {
    fn abi_test_eabi_probe();

    #[link_name = "i8"]
    fn eabi_i8(
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
        h: u32,
        ptr: unsafe extern "C" fn(),
    ) -> u32;

    #[link_name = "i9"]
    fn eabi_i9(
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
        h: u32,
        i: u32,
        ptr: unsafe extern "C" fn(),
    ) -> u32;
}

fn seen() -> [u32; 9] {
    unsafe { addr_of!(SEEN).read_volatile() }
}

fn trampolines(test_runner: &mut TestRunner) {
    unsafe {
        addr_of_mut!(SEEN).write_volatile([0; 9]);
        let r = eabi_i8(1, 2, 3, 4, 5, 6, 7, 8, abi_test_eabi_probe);
        test_runner.check("abi_i8_return", r, 0x600d);
        test_runner.check("abi_i8_args", seen()[..8].to_vec(), (1..=8).collect());

        addr_of_mut!(SEEN).write_volatile([0; 9]);
        let r = eabi_i9(1, 2, 3, 4, 5, 6, 7, 8, 9, abi_test_eabi_probe);
        test_runner.check("abi_i9_return", r, 0x600d);
        test_runner.check("abi_i9_args", seen().to_vec(), (1..=9).collect());
    }
}
