//! The run-time loader of the system libraries this crate wraps (FreeType, fontconfig;
//! visualization-design.md section 2): `dlopen` once, `dlsym` per entry point, never closed.
//! Windows uses LoadLibraryW/GetProcAddress and retains the same process lifetime.
#![allow(unsafe_code)]
#[cfg(any(unix, windows))]
use std::ffi::CString;
use std::ffi::c_void;

/// An open shared library; `DynamicLibrary::open(&["libfreetype.so.6"])`.
pub(crate) struct DynamicLibrary {
    /// The `dlopen` handle (an address; the library stays loaded for the process).
    #[cfg(any(unix, windows))]
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

    #[cfg(windows)]
    fn open_one(name: &str) -> Option<Self> {
        if name.contains('\0') {
            return None;
        }
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        // SAFETY: the terminated path lives through the call; retain the DLL for its symbols.
        let handle =
            unsafe { windows_sys::Win32::System::LibraryLoader::LoadLibraryW(wide.as_ptr()) };
        (!handle.is_null()).then_some(Self {
            handle: handle as usize,
        })
    }

    #[cfg(not(any(unix, windows)))]
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

    #[cfg(windows)]
    pub(crate) fn symbol(&self, name: &str) -> Option<*mut c_void> {
        let name = CString::new(name).ok()?;
        // SAFETY: this DLL remains loaded and the symbol name is terminated.
        let address = unsafe {
            windows_sys::Win32::System::LibraryLoader::GetProcAddress(
                self.handle as *mut c_void,
                name.as_ptr().cast(),
            )
        }?;
        Some(address as *mut c_void)
    }

    #[cfg(not(any(unix, windows)))]
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

#[cfg(test)]
mod tests {
    use super::DynamicLibrary;
    #[test]
    fn missing_libraries_and_invalid_names_are_rejected() {
        assert!(DynamicLibrary::open(&["Martensite-no-such-library-96c1"]).is_none());
        assert!(DynamicLibrary::open(&["bad\0name"]).is_none());
    }
    #[cfg(windows)]
    #[test]
    fn windows_system_dll_exports_are_retained_and_resolved() {
        let dll = DynamicLibrary::open(&["missing-96c1.dll", "kernel32.dll"]).unwrap();
        assert!(dll.symbol("GetCurrentProcessId").is_some());
        assert!(dll.symbol("Martensite-no-such-symbol-96c1").is_none());
        assert!(dll.symbol("bad\0symbol").is_none());
    }
}
