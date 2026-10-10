//! The run-time loader of the system libraries this crate wraps (FreeType, fontconfig;
//! visualization-design.md section 2): `dlopen` once, `dlsym` per entry point, never closed.
//! On a platform without `dlopen` every library is reported as missing.
#![allow(unsafe_code)]
#[cfg(unix)]
use std::ffi::CString;
use std::ffi::c_void;

/// An open shared library; `DynamicLibrary::open(&["libfreetype.so.6"])`.
pub(crate) struct DynamicLibrary {
    /// The `dlopen` handle (an address; the library stays loaded for the process).
    #[cfg(unix)]
    handle: usize,
}

impl DynamicLibrary {
    /// Opens the first library of `names` that loads, or `None`.
    pub(crate) fn open(names: &[&str]) -> Option<Self> {
        names.iter().find_map(|name| Self::open_one(name))
    }

    #[cfg(unix)]
    fn open_one(name: &str) -> Option<Self> {
        let name = CString::new(name).ok()?;
        // SAFETY: dlopen reads a NUL-terminated name; the returned handle (or null) is kept
        // for the process lifetime and never closed, so the symbols stay valid.
        let handle = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
        (!handle.is_null()).then_some(Self {
            handle: handle as usize,
        })
    }

    #[cfg(not(unix))]
    fn open_one(_name: &str) -> Option<Self> {
        None
    }

    /// The address of the entry point `name`, or `None` when the library lacks it.
    #[cfg(unix)]
    pub(crate) fn symbol(&self, name: &str) -> Option<*mut c_void> {
        let name = CString::new(name).ok()?;
        // SAFETY: the handle comes from a successful dlopen that is never closed; dlsym
        // reads a NUL-terminated name.
        let address = unsafe { libc::dlsym(self.handle as *mut c_void, name.as_ptr()) };
        (!address.is_null()).then_some(address)
    }

    #[cfg(not(unix))]
    pub(crate) fn symbol(&self, _name: &str) -> Option<*mut c_void> {
        None
    }
}

/// Reinterprets a symbol address as the function pointer type `T` (pointer-sized,
/// checked at compile time).
///
/// # Safety
/// `address` is the entry point of a C function whose signature is `T`.
pub(crate) unsafe fn cast_symbol<T: Copy>(address: *mut c_void) -> T {
    const { assert!(std::mem::size_of::<T>() == std::mem::size_of::<*mut c_void>()) };
    // SAFETY: same size (checked above); the caller guarantees the signature.
    unsafe { std::mem::transmute_copy::<*mut c_void, T>(&address) }
}

/// Binds the named entry points of a library into a struct of `extern "C"` function
/// pointers: `bind_symbols!(lib, Api { FT_Init_FreeType: fn(...) -> i32, ... })`.
/// The whole struct is `None` when any entry point is missing.
macro_rules! bind_symbols {
    ($lib:expr, $api:ident { $($name:ident),+ $(,)? }) => {{
        let lib = $lib;
        (|| {
            Some($api {
                $(
                    // SAFETY: the symbol is the C function of that name, whose signature
                    // the field type declares as in the library's public header.
                    $name: unsafe { $crate::dynlib::cast_symbol(lib.symbol(stringify!($name))?) },
                )+
            })
        })()
    }};
}
pub(crate) use bind_symbols;
