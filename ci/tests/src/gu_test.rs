use core::ffi::c_void;
use psp::sys::{self, GuContextType, MipmapLevel};
use psp::test_runner::TestRunner;

#[repr(C, align(64))]
struct List([u32; 256]);

static mut LIST: List = List([0; 256]);

/// The `TSIZE` argument `sceGuTexImage` writes for a `width` x `height`
/// texture, read back from a call list (built in memory, never run).
fn tex_size(width: i32, height: i32) -> Option<u32> {
    unsafe {
        let list = core::ptr::addr_of_mut!(LIST) as *mut u32;
        sys::sceGuStart(GuContextType::Call, list as *mut c_void);
        sys::sceGuTexImage(MipmapLevel::None, width, height, width, core::ptr::null());
        sys::sceGuFinish();
        // Read through the uncached alias, which sceGuStart writes through.
        let words = ((list as u32) | 0x4000_0000) as *const u32;
        (0..256)
            .map(|i| words.add(i).read_volatile())
            .find(|w| w >> 24 == 0xb8)
            .map(|w| w & 0xff_ffff)
    }
}

pub fn test_main(test_runner: &mut TestRunner) {
    test_runner.check("gu_tex_size_512", tex_size(512, 512), Some(0x909));
    test_runner.check("gu_tex_size_64x1", tex_size(64, 1), Some(0x006));
    // Over 512 is declared as 512, the most the GE can address.
    test_runner.check("gu_tex_size_1024x64", tex_size(1024, 64), Some(0x609));
    test_runner.check("gu_tex_size_16x1024", tex_size(16, 1024), Some(0x904));
}
