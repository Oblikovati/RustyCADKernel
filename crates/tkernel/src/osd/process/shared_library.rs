//! `OSD_SharedLibrary`: load a shared library at run time and look up its symbols.
//!
//! The Unix build is a thin layer over `dlopen`/`dlsym`/`dlclose`/`dlerror`, reached
//! through narrow platform modules below. Windows uses the OCCT `GetModuleHandleW` cache,
//! `LoadLibraryExW`, and system-error formatter; its runtime behavior is cross-compile and
//! fake-tested here but not executed on Windows. Byte methods preserve Unix loader names
//! and errors that are not valid UTF-8; string methods are convenience views.
use crate::{
    osd::LoadMode,
    standard::{c_prefix, c_string::c_str_prefix},
};
use std::{ffi::c_void, ptr::NonNull};

#[cfg(any(windows, test))]
mod windows_backend;

/// The address `DlSymb` returns (`OSD_Function`, a C function pointer); callers cast it to
/// the real signature with `std::mem::transmute`.
pub type SymbolAddress = NonNull<c_void>;

/// `OSD_SharedLibrary`; `SharedLibrary::named(Some("libm.so.6")).dl_open(LoadMode::RtldNow)`.
#[derive(Debug)]
pub struct SharedLibrary {
    name: Option<Vec<u8>>,
    handle: *mut c_void,
    // Set by dl_close: native keeps a dangling handle, and a later DlSymb is undefined.
    closed: bool,
}
impl Default for SharedLibrary {
    fn default() -> Self {
        Self::new()
    }
}
impl SharedLibrary {
    /// `OSD_SharedLibrary()`: no name and a null handle; `SharedLibrary::new()`.
    /// Opening a null name is platform-specific (Unix `dlopen(NULL)`; OCCT Windows passes
    /// the missing name into its wide-string conversion path).
    pub const fn new() -> Self {
        Self {
            name: None,
            handle: std::ptr::null_mut(),
            closed: false,
        }
    }
    /// `OSD_SharedLibrary(const char*)`; `None` stands for a null name;
    /// `SharedLibrary::named(Some("libc.so.6"))`.
    pub fn named(name: Option<&str>) -> Self {
        Self::named_bytes(name.map(|value| c_prefix(value).as_bytes()))
    }
    /// Stores an OS-native byte name; `SharedLibrary::named_bytes(Some(b"libm.so.6"))`.
    /// Bytes after the first NUL are ignored, as in the native `const char*` constructor.
    pub fn named_bytes(name: Option<&[u8]>) -> Self {
        let mut library: Self = Self::new();
        library.set_name_bytes(name);
        library
    }
    /// `OSD_SharedLibrary::SetName`: a null name keeps the current one;
    /// `library.set_name(Some("libm.so.6\0ignored"))` stores `libm.so.6` like native `strlen`.
    pub fn set_name(&mut self, name: Option<&str>) {
        self.set_name_bytes(name.map(|value| c_prefix(value).as_bytes()));
    }
    /// Replaces the name with native bytes; `library.set_name_bytes(Some(b"libm.so.6"))`.
    pub fn set_name_bytes(&mut self, name: Option<&[u8]>) {
        if let Some(name) = name {
            self.name = Some(c_str_prefix(name).to_vec());
            #[cfg(windows)]
            {
                self.handle = ffi::cached_handle(self.name.as_deref().unwrap_or_default());
                self.closed = false;
            }
        }
    }
    /// Returns a UTF-8 view of the name, or `None` for null/invalid bytes; `library.name()`.
    pub fn name(&self) -> Option<&str> {
        self.name
            .as_deref()
            .and_then(|name| std::str::from_utf8(name).ok())
    }
    /// Returns the stored OS-native bytes; `library.name_bytes() == Some(b"libm.so.6")`.
    pub fn name_bytes(&self) -> Option<&[u8]> {
        self.name.as_deref()
    }
    /// `OSD_SharedLibrary::DlOpen`: loads the library (`RTLD_LAZY` or `RTLD_NOW`) and
    /// tells whether a handle was obtained; `library.dl_open(LoadMode::RtldLazy)`.
    pub fn dl_open(&mut self, mode: LoadMode) -> bool {
        #[cfg(windows)]
        let was_closed: bool = self.closed;
        self.handle = ffi::open(self.name.as_deref(), mode, self.handle);
        #[cfg(not(windows))]
        {
            self.closed = false;
        }
        #[cfg(windows)]
        {
            self.closed = was_closed;
        }
        !self.handle.is_null()
    }
    /// `OSD_SharedLibrary::DlSymb`: the address of a symbol, `None` when absent;
    /// `library.dl_symb("cos")`.
    ///
    /// Before `dl_open` the handle is null, which glibc reads as `RTLD_DEFAULT` (the
    /// global scope), as natively. After `dl_close` Rust answers `None` instead of the
    /// native use of a dangling handle.
    pub fn dl_symb(&self, name: &str) -> Option<SymbolAddress> {
        self.dl_symb_bytes(c_prefix(name).as_bytes())
    }
    /// Resolves an OS-native symbol name; `library.dl_symb_bytes(b"cos")`.
    pub fn dl_symb_bytes(&self, name: &[u8]) -> Option<SymbolAddress> {
        if self.closed {
            return None;
        }
        NonNull::new(ffi::symbol(self.handle, c_str_prefix(name)))
    }
    /// `OSD_SharedLibrary::DlClose`: unloads the library once; `library.dl_close()`.
    ///
    /// Native `dlclose(NULL)` (close before open) and double closes are undefined; Rust
    /// ignores them.
    pub fn dl_close(&mut self) {
        if self.closed || self.handle.is_null() {
            return;
        }
        ffi::close(self.handle);
        self.closed = true;
    }
    /// Returns loader diagnostics as lossy UTF-8; use `dl_error_bytes()` for raw bytes.
    /// Unix consumes its pending loader error; Windows formats the retained last error code.
    /// Example: `library.dl_error()`.
    pub fn dl_error(&self) -> Option<String> {
        self.dl_error_bytes()
            .map(|error| String::from_utf8_lossy(&error).into_owned())
    }
    /// Reads loader error bytes; `library.dl_error_bytes()` preserves non-UTF-8 diagnostics.
    /// Unix consumes its pending error, while Windows retains the last error code.
    pub fn dl_error_bytes(&self) -> Option<Vec<u8>> {
        ffi::last_error(self.name.as_deref())
    }
    /// `OSD_SharedLibrary::Destroy` (also the destructor): forgets the name and the handle
    /// without unloading the library; `library.destroy()`.
    pub fn destroy(&mut self) {
        self.name = None;
        #[cfg(not(windows))]
        {
            self.handle = std::ptr::null_mut();
            self.closed = false;
        }
    }
}

#[cfg(unix)]
#[allow(unsafe_code)]
mod ffi {
    use crate::osd::LoadMode;
    use crate::standard::c_string::c_str_prefix;
    use std::ffi::{CStr, CString, c_void};

    pub(super) fn open(name: Option<&[u8]>, mode: LoadMode, _current: *mut c_void) -> *mut c_void {
        let flags: i32 = match mode {
            LoadMode::RtldLazy => libc::RTLD_LAZY,
            LoadMode::RtldNow => libc::RTLD_NOW,
        };
        let Some(name) = name else {
            // SAFETY: a null file name asks for the main program handle.
            return unsafe { libc::dlopen(std::ptr::null(), flags) };
        };
        let Ok(name) = CString::new(c_str_prefix(name)) else {
            return std::ptr::null_mut();
        };
        // SAFETY: `name` is a valid C string for the duration of the call.
        unsafe { libc::dlopen(name.as_ptr(), flags) }
    }

    pub(super) fn symbol(handle: *mut c_void, name: &[u8]) -> *mut c_void {
        let Ok(name) = CString::new(c_str_prefix(name)) else {
            return std::ptr::null_mut();
        };
        // SAFETY: `handle` is null (global scope) or a live dlopen handle; `name` is a
        // valid C string.
        unsafe { libc::dlsym(handle, name.as_ptr()) }
    }

    pub(super) fn close(handle: *mut c_void) {
        // SAFETY: the caller passes a non-null handle not closed before.
        unsafe { libc::dlclose(handle) };
    }

    pub(super) fn last_error(_name: Option<&[u8]>) -> Option<Vec<u8>> {
        // SAFETY: dlerror returns null or a NUL-terminated message owned by the loader,
        // copied before any other loader call on this thread.
        let text: *const libc::c_char = unsafe { libc::dlerror() };
        // SAFETY: non-null per the check.
        (!text.is_null()).then(|| unsafe { CStr::from_ptr(text) }.to_bytes().to_vec())
    }
}

#[cfg(windows)]
mod ffi {
    use super::windows_backend::{
        SystemWindowsCalls, WindowsErrorBuffers, close_with, error_with, module_cache_with,
        open_with, symbol_with,
    };
    use crate::osd::LoadMode;
    use std::{
        ffi::c_void,
        sync::{Mutex, atomic::AtomicU32},
    };

    static CALLS: SystemWindowsCalls = SystemWindowsCalls;
    static LAST_DLL_ERROR: AtomicU32 = AtomicU32::new(0);
    static ERROR_BUFFERS: Mutex<WindowsErrorBuffers> = Mutex::new(WindowsErrorBuffers::new());

    pub(super) fn cached_handle(name: &[u8]) -> *mut c_void {
        module_cache_with(&CALLS, name)
    }
    pub(super) fn open(name: Option<&[u8]>, _mode: LoadMode, current: *mut c_void) -> *mut c_void {
        open_with(&CALLS, name, current, &LAST_DLL_ERROR)
    }
    pub(super) fn symbol(handle: *mut c_void, name: &[u8]) -> *mut c_void {
        symbol_with(&CALLS, handle, name, &LAST_DLL_ERROR)
    }
    pub(super) fn close(handle: *mut c_void) {
        close_with(&CALLS, handle);
    }
    pub(super) fn last_error(name: Option<&[u8]>) -> Option<Vec<u8>> {
        Some(error_with(&CALLS, &LAST_DLL_ERROR, &ERROR_BUFFERS, name))
    }
}

#[cfg(not(any(unix, windows)))]
mod ffi {
    use crate::osd::LoadMode;
    use std::ffi::c_void;

    pub(super) fn open(
        _name: Option<&[u8]>,
        _mode: LoadMode,
        _current: *mut c_void,
    ) -> *mut c_void {
        std::ptr::null_mut()
    }
    pub(super) fn symbol(_handle: *mut c_void, _name: &[u8]) -> *mut c_void {
        std::ptr::null_mut()
    }
    pub(super) fn close(_handle: *mut c_void) {}
    pub(super) fn last_error(_name: Option<&[u8]>) -> Option<Vec<u8>> {
        Some(b"OSD_SharedLibrary: the Windows loader is not ported".to_vec())
    }
}
