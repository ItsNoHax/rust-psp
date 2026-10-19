#[cfg(target_os = "psp")]
use crate::sys::SceStubLibraryEntry;

// `psp_extern!` expands in other crates too, which reach `paste!` here.
#[doc(hidden)]
pub use paste::paste;

/// A macro that enables the use of `concat!` inside the `#[link_section = ...]`
/// attribute.
#[doc(hidden)]
#[macro_export]
macro_rules! link_section_concat {
    ($(#[link_section = $section:expr] $item:item)*) => {
        $(
            #[link_section = $section]
            $item
        )*
    }
}

/// A "function" stub.
///
/// This is a very dirty trick for LTO. Essentially, the PSP OS takes the
/// address of a stub and inserts 2 instructions (8 bytes) which are `jr $ra`
/// and `syscall xxx`. Traditionally, the C/C++ toolchain stores the initial
/// data here as just an empty function that immediately returns (8 bytes,
/// `jr $ra; nop`). However, we use it to store 2 references: to the NID and to
/// the library stub.
///
/// This results in LTO builds compiling in the NID and library stubs whenever a
/// function is called, as this struct references them. Thus, this struct
/// definition creates a dependency between the function call, and the NID + lib
/// stub. As mentioned earlier, these two addresses (8 bytes) are replaced with
/// two instructions (also 8 bytes) at runtime, so they are not actually called
/// as a function. With this method, nothing has to be marked `#[used]`, so LLVM
/// can automatically remove unreferenced NIDs and library stubs during LTO.
/// Compiling with LTO then only links the functions that are called, and no
/// more.
#[cfg(target_os = "psp")]
#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct Stub {
    // These are never read, but need to be written into as static items.
    #[allow(dead_code)]
    pub lib_addr: &'static SceStubLibraryEntry,
    #[allow(dead_code)]
    pub nid_addr: &'static u32,
}

/// Calculate the padded length for a library name.
///
/// The name is padded on the end with zeroes. Must be at least one and a
/// multiple of 4.
#[cfg(target_os = "psp")]
pub const fn lib_name_bytes_len(name: &str) -> usize {
    let name_len = name.as_bytes().len();
    return name_len + (4 - name_len % 4);
}

/// Convert a library name to a byte array.
///
/// This is intended to be used with `lib_name_bytes_len`.
#[cfg(target_os = "psp")]
pub const fn lib_name_bytes<const T: usize>(name: &str) -> [u8; T] {
    let mut buf = [0; T];

    let name_bytes = name.as_bytes();
    let mut i = 0;

    while i < name_bytes.len() {
        buf[i] = name_bytes[i];
        i += 1;
    }

    buf
}

/// Imports functions from a PSP system library by NID.
///
/// rust-psp binds the standard libraries with this; an application can bind
/// one it does not, such as a custom firmware's. `name` is the library's
/// export name and `flags` its stub flags (`0x4001` for a user-mode import,
/// `0x4009` for a syscall export). Each function gets a stub that the loader
/// replaces with a `jr $ra; syscall` to the NID, and is exported unmangled
/// under its own name.
///
/// ```ignore
/// psp::psp_extern! {
///     #![name = "KUBridge"]
///     #![flags = 0x4001]
///     #![version = (0x00, 0x00)]
///
///     #[psp(0x7A50075E)]
///     pub fn kuKernelPeekw(addr: u32) -> u32;
/// }
/// ```
#[macro_export]
macro_rules! psp_extern {
    // Generate body with default ABI.
    (__BODY $name:ident ($($arg:ident : $arg_ty:ty),*) $(-> $ret:ty)?) => {{
        $crate::sys::macros::paste! {
            extern "C" {
                pub fn [< __ $name _stub >]($($arg : $arg_ty),*) $(-> $ret)?;
            }
            let func = [< __ $name _stub >];
            func($($arg),*)
        }
    }};

    // Generate body with an ABI mapper
    (__BODY $abi:ident $name:ident ($($arg:ident : $arg_ty:ty),*) $(-> $ret:ty)?) => {{
        type Func = unsafe extern "C" fn($($arg : $arg_ty),*) $(-> $ret)?;

        $crate::sys::macros::paste! {
            extern "C" {
                pub fn [< __ $name _stub >]($($arg : $arg_ty),*) $(-> $ret)?;
            }
            let func = [< __ $name _stub >] as Func;

            // The transmutes here are for newtypes that fit into a single
            // register.
            core::mem::transmute($abi(
                $(core::mem::transmute($arg)),*,
                core::mem::transmute(func),
            ))
        }
    }};

    (
        #![name = $lib_name:expr]
        #![flags = $lib_flags:expr]
        #![version = ($lib_major_version:expr, $lib_minor_version:expr)]

        $(
            #[psp($nid:expr $(, $abi:ident)?)]
            $(#[$attr:meta])*
            pub fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?)
            $(-> $ret:ty)?;
        )*
    ) => {
        $crate::sys::macros::paste! {
            #[allow(non_snake_case)]
            mod [< __ $lib_name _mod >] {
                #[allow(unused)]
                use super::*;

                #[cfg(target_os = "psp")]
                $crate::link_section_concat! {
                    #[link_section = concat!(".rodata.sceResident.", $lib_name)]
                    #[allow(non_upper_case_globals)]
                    static [< __ $lib_name _RESIDENT >] : [u8; $crate::sys::macros::lib_name_bytes_len($lib_name)] = $crate::sys::macros::lib_name_bytes($lib_name);

                    #[link_section = concat!(".rodata.sceNid.", $lib_name)]
                    #[allow(non_upper_case_globals)]
                    static [< __ $lib_name _NID_START >] : () = ();

                    #[link_section = concat!(".sceStub.text.", $lib_name)]
                    #[allow(non_upper_case_globals)]
                    static [< __ $lib_name _STUB_START >] : () = ();

                    #[link_section = concat!(".lib.stub.entry.", $lib_name)]
                    #[allow(non_upper_case_globals)]
                    static [< __ $lib_name _STUB >] : $crate::sys::SceStubLibraryEntry = $crate::sys::SceStubLibraryEntry {
                        name: $crate::sys::macros::paste! { & [< __ $lib_name _RESIDENT >] [0] },
                        version: [$lib_minor_version, $lib_major_version],
                        flags: $lib_flags,
                        len: 5,
                        v_stub_count: 0,

                        // This is to be fixed up later by patching the ELF file,
                        // in cargo-psp. The reason we cannot determine this
                        // statically is that the real number of imported stubs
                        // is found during link time.
                        //
                        // For example, in LTO builds, imports can be removed at link
                        // time if they are not ever called. There may be (?) a way
                        // to get the linker to handle this, but I have not found a
                        // working method.
                        stub_count: 0,

                        nid_table: & [< __ $lib_name _NID_START >] as *const () as *const _,
                        stub_table: & [< __ $lib_name _STUB_START >] as *const () as *const _,
                    };
                }

                $(
                    #[allow(unused)]
                    use super::*;

                    #[cfg(target_os = "psp")]
                    $crate::link_section_concat! {
                        #[link_section = concat!(
                            ".rodata.sceNid.", $lib_name,
                            ".", stringify!($name)
                        )]
                        #[allow(non_upper_case_globals)]
                        static [< __ $name _NID >]: u32 = $nid;

                        #[link_section = concat!(
                            ".sceStub.text.", $lib_name,
                            ".", stringify!($name)
                        )]
                        #[no_mangle]
                        #[allow(non_upper_case_globals)]
                        static [< __ $name _stub >]: $crate::sys::macros::Stub = $crate::sys::macros::Stub {
                            lib_addr: &[< __ $lib_name _STUB >],
                            nid_addr: &[< __ $name _NID >],
                        };
                    }

                    $(#[$attr])*
                    #[allow(non_snake_case, clippy::missing_safety_doc)]
                    #[no_mangle]
                    pub unsafe extern "C" fn $name($($arg : $arg_ty),*) $(-> $ret)? {
                        #[cfg(target_os = "psp")]
                        {
                            $crate::psp_extern!(
                                __BODY $($abi)?
                                $name($($arg : $arg_ty),*) $(-> $ret)?
                            )
                        }

                        #[cfg(not(target_os = "psp"))]
                        {
                            // Get rid of warnings
                            $(let _arg = $arg;)*
                            $(let _abi = $abi;)?

                            panic!("tried to call PSP system function on non-PSP target");
                        }
                    }
                )*
            }
        }

        $crate::sys::macros::paste! {
            $(
                pub use self :: [< __ $lib_name _mod >] :: $name;
            )*
        }
    }
}
