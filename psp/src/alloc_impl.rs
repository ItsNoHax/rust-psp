use crate::sys::{self, SceSysMemBlockTypes, SceSysMemPartitionId, SceUid};
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};

/// An allocator that hooks directly into the PSP OS memory allocator: every
/// allocation and free is a kernel call. The global allocator unless the
/// `external-allocator` feature is enabled.
pub struct SystemAlloc;

unsafe impl GlobalAlloc for SystemAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size()
            // We need to store the memory block ID.
            + mem::size_of::<SceUid>()

            // We also store padding bytes, in case the block returned from the
            // system is not aligned. The count of padding bytes is also stored
            // here, in the last byte.
            + layout.align();

        let id = sys::sceKernelAllocPartitionMemory(
            SceSysMemPartitionId::SceKernelPrimaryUserPartition,
            &b"block\0"[0],
            SceSysMemBlockTypes::Low,
            size as u32,
            ptr::null_mut(),
        );

        if id.0 < 0 {
            return ptr::null_mut();
        }

        let mut ptr: *mut u8 = sys::sceKernelGetBlockHeadAddr(id).cast();
        *ptr.cast() = id;

        ptr = ptr.add(mem::size_of::<SceUid>());

        // We must add at least one, to store this value.
        let align_padding = 1 + ptr.add(1).align_offset(layout.align());
        *ptr.add(align_padding - 1) = align_padding as u8;
        ptr.add(align_padding)
    }

    #[inline(never)]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let align_padding = *ptr.sub(1);

        let id = *ptr.sub(align_padding as usize).cast::<SceUid>().offset(-1);

        sys::sceKernelFreePartitionMemory(id);
    }
}

#[cfg(not(feature = "external-allocator"))]
#[global_allocator]
static ALLOC: SystemAlloc = SystemAlloc;

/// With the `external-allocator` feature the global allocator forwards to
/// three functions the application defines with `#[no_mangle]`:
///
/// ```ignore
/// #[no_mangle]
/// unsafe fn __psp_external_alloc(size: usize, align: usize) -> *mut u8;
/// #[no_mangle]
/// unsafe fn __psp_external_dealloc(ptr: *mut u8, size: usize, align: usize);
/// #[no_mangle]
/// unsafe fn __psp_external_realloc(ptr: *mut u8, size: usize, align: usize, new_size: usize) -> *mut u8;
/// ```
///
/// They may fall back on [`SystemAlloc`]. (A `#[global_allocator]` in the
/// application itself would leave this crate's `staticlib` without one.)
#[cfg(feature = "external-allocator")]
mod external {
    use alloc::alloc::{GlobalAlloc, Layout};

    extern "Rust" {
        fn __psp_external_alloc(size: usize, align: usize) -> *mut u8;
        fn __psp_external_dealloc(ptr: *mut u8, size: usize, align: usize);
        fn __psp_external_realloc(
            ptr: *mut u8,
            size: usize,
            align: usize,
            new_size: usize,
        ) -> *mut u8;
    }

    struct ExternalAlloc;

    unsafe impl GlobalAlloc for ExternalAlloc {
        #[inline]
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            __psp_external_alloc(layout.size(), layout.align())
        }

        #[inline]
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            __psp_external_dealloc(ptr, layout.size(), layout.align())
        }

        #[inline]
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            __psp_external_realloc(ptr, layout.size(), layout.align(), new_size)
        }
    }

    #[global_allocator]
    static ALLOC: ExternalAlloc = ExternalAlloc;
}

#[cfg(not(feature = "std"))]
#[alloc_error_handler]
fn aeh(_: Layout) -> ! {
    loop {
        core::hint::spin_loop()
    }
}

#[no_mangle]
#[cfg(not(feature = "stub-only"))]
unsafe extern "C" fn memset(ptr: *mut u8, value: i32, num: usize) -> *mut u8 {
    crate::mem::fill(ptr, value as u8, num);
    ptr
}

#[no_mangle]
#[cfg(not(feature = "stub-only"))]
unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, num: usize) -> *mut u8 {
    crate::mem::copy_forward(dst, src, num);
    dst
}

#[no_mangle]
#[cfg(not(feature = "stub-only"))]
unsafe extern "C" fn memcmp(ptr1: *mut u8, ptr2: *mut u8, num: usize) -> i32 {
    let mut i = 0;

    while i < num {
        let val1 = *((ptr1 as usize + i) as *mut u8);
        let val2 = *((ptr2 as usize + i) as *mut u8);
        let diff = val1 as i32 - val2 as i32;

        if diff != 0 {
            return diff;
        }

        i += 1;
    }

    0
}

#[no_mangle]
#[cfg(not(feature = "stub-only"))]
unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, num: usize) -> *mut u8 {
    crate::mem::move_bytes(dst, src, num);
    dst
}

#[no_mangle]
#[cfg(not(feature = "stub-only"))]
unsafe extern "C" fn strlen(s: *mut u8) -> usize {
    let mut len = 0;

    while *s.add(len) != 0 {
        len += 1;
    }

    len
}
