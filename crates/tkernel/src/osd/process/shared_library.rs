//! `OSD_SharedLibrary`: load a shared library at run time and look up its symbols.
//!
//! The Unix build is a thin layer over `dlopen`/`dlsym`/`dlclose`/`dlerror`, reached
//! through the narrow FFI module below. The Windows build (`LoadLibraryExW`,
//! `GetProcAddress`) needs loader bindings the crate does not link yet: there `dl_open`
//! fails and `dl_error` says so.
use crate::osd::LoadMode;
use std::{ffi::c_void, ptr::NonNull};

/// The address `DlSymb` returns (`OSD_Function`, a C function pointer); callers cast it to
/// the real signature with `std::mem::transmute`.
pub type SymbolAddress = NonNull<c_void>;

/// `OSD_SharedLibrary`; `SharedLibrary::named(Some("libm.so.6")).dl_open(LoadMode::RtldNow)`.
#[derive(Debug)]
pub struct SharedLibrary {
    name: Option<String>,
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
    /// `OSD_SharedLibrary()`: no name; opening it yields the main program; `SharedLibrary::new()`.
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
        let mut library: Self = Self::new();
        library.set_name(name);
        library
    }
    /// `OSD_SharedLibrary::SetName`: a null name keeps the current one;
    /// `library.set_name(Some("libm.so.6"))`.
    pub fn set_name(&mut self, name: Option<&str>) {
        if let Some(name) = name {
            self.name = Some(name.to_string());
        }
    }
    /// `OSD_SharedLibrary::Name`: the stored name, `None` for none; `library.name()`.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    /// `OSD_SharedLibrary::DlOpen`: loads the library (`RTLD_LAZY` or `RTLD_NOW`) and
    /// tells whether a handle was obtained; `library.dl_open(LoadMode::RtldLazy)`.
    pub fn dl_open(&mut self, mode: LoadMode) -> bool {
        self.handle = ffi::open(self.name.as_deref(), mode);
        self.closed = false;
        !self.handle.is_null()
    }
    /// `OSD_SharedLibrary::DlSymb`: the address of a symbol, `None` when absent;
    /// `library.dl_symb("cos")`.
    ///
    /// Before `dl_open` the handle is null, which glibc reads as `RTLD_DEFAULT` (the
    /// global scope), as natively. After `dl_close` Rust answers `None` instead of the
    /// native use of a dangling handle.
    pub fn dl_symb(&self, name: &str) -> Option<SymbolAddress> {
        if self.closed {
            return None;
        }
        NonNull::new(ffi::symbol(self.handle, name))
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
    /// `OSD_SharedLibrary::DlError`: the last loader error text, consumed by reading it
    /// (`dlerror` semantics), `None` when there is none; `library.dl_error()`.
    pub fn dl_error(&self) -> Option<String> {
        ffi::last_error()
    }
    /// `OSD_SharedLibrary::Destroy` (also the destructor): forgets the name and the handle
    /// without unloading the library; `library.destroy()`.
    pub fn destroy(&mut self) {
        self.name = None;
        self.handle = std::ptr::null_mut();
        self.closed = false;
    }
}

#[cfg(unix)]
#[allow(unsafe_code)]
mod ffi {
    use crate::osd::LoadMode;
    use std::ffi::{CStr, CString, c_void};

    pub(super) fn open(name: Option<&str>, mode: LoadMode) -> *mut c_void {
        let flags: i32 = match mode {
            LoadMode::RtldLazy => libc::RTLD_LAZY,
            LoadMode::RtldNow => libc::RTLD_NOW,
        };
        let Some(name) = name else {
            // SAFETY: a null file name asks for the main program handle.
            return unsafe { libc::dlopen(std::ptr::null(), flags) };
        };
        let Ok(name) = CString::new(name) else {
            return std::ptr::null_mut();
        };
        // SAFETY: `name` is a valid C string for the duration of the call.
        unsafe { libc::dlopen(name.as_ptr(), flags) }
    }

    pub(super) fn symbol(handle: *mut c_void, name: &str) -> *mut c_void {
        let Ok(name) = CString::new(name) else {
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

    pub(super) fn last_error() -> Option<String> {
        // SAFETY: dlerror returns null or a NUL-terminated message owned by the loader,
        // copied before any other loader call on this thread.
        let text: *const libc::c_char = unsafe { libc::dlerror() };
        // SAFETY: non-null per the check.
        (!text.is_null()).then(|| {
            unsafe { CStr::from_ptr(text) }
                .to_string_lossy()
                .into_owned()
        })
    }
}

#[cfg(not(unix))]
mod ffi {
    use crate::osd::LoadMode;
    use std::ffi::c_void;

    pub(super) fn open(_name: Option<&str>, _mode: LoadMode) -> *mut c_void {
        std::ptr::null_mut()
    }
    pub(super) fn symbol(_handle: *mut c_void, _name: &str) -> *mut c_void {
        std::ptr::null_mut()
    }
    pub(super) fn close(_handle: *mut c_void) {}
    pub(super) fn last_error() -> Option<String> {
        Some("OSD_SharedLibrary: the Windows loader is not ported".to_string())
    }
}
