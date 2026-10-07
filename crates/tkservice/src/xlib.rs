//! The thin project-owned wrapper of Xlib (visualization-design.md section 2).
//!
//! `x11-dl` loads `libX11.so.6` at run time, so the crate builds without the X11
//! development files. Only the calls that Aspect_DisplayConnection and Xw_Window make are
//! exposed; pointers cross this module as addresses (`usize`), and a missing library is
//! reported as the failure of the call that needed it.
#![allow(unsafe_code)]
use std::ffi::{CStr, c_void};
use std::sync::OnceLock;
use x11_dl::xlib::{Display, Xlib};

/// The loaded Xlib, or `None` when `libX11.so.6` cannot be loaded.
pub(crate) fn xlib() -> Option<&'static XlibApi> {
    static LOADED: OnceLock<Option<XlibApi>> = OnceLock::new();
    LOADED
        .get_or_init(|| Xlib::open().ok().map(|lib| XlibApi { lib }))
        .as_ref()
}

/// The Xlib entry points this crate calls.
pub(crate) struct XlibApi {
    lib: Xlib,
}

impl XlibApi {
    /// `XOpenDisplay(name)`: the display address, 0 when the server cannot be reached.
    pub(crate) fn open_display(&self, name: &CStr) -> usize {
        // SAFETY: XOpenDisplay reads a NUL-terminated string and returns a new display
        // or null; `name` outlives the call.
        unsafe { (self.lib.XOpenDisplay)(name.as_ptr()) as usize }
    }
    /// `XCloseDisplay(display)` for a display this crate opened.
    pub(crate) fn close_display(&self, display: usize) {
        // SAFETY: callers pass only addresses returned by `open_display` and close each
        // of them once (Aspect_DisplayConnection tracks ownership).
        unsafe {
            (self.lib.XCloseDisplay)(display as *mut Display);
        }
    }
    /// `XInternAtom(display, name, only_if_exists)`: the atom id.
    pub(crate) fn intern_atom(&self, display: usize, name: &CStr, only_if_exists: bool) -> u64 {
        // SAFETY: `display` is a live display (opened here or handed in by the caller as
        // natively), and `name` is NUL-terminated.
        let atom = unsafe {
            (self.lib.XInternAtom)(
                display as *mut Display,
                name.as_ptr(),
                i32::from(only_if_exists),
            )
        };
        // Atom is c_ulong: u64 on 64-bit Linux, u32 on 32-bit Linux.
        #[allow(clippy::unnecessary_cast)]
        let atom = atom as u64;
        atom
    }
    /// `XFree(data)` of memory that Xlib allocated.
    pub(crate) fn free(&self, data: usize) {
        // SAFETY: callers pass only addresses that Xlib allocated (visual infos).
        unsafe {
            (self.lib.XFree)(data as *mut c_void);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::xlib;

    #[test]
    fn an_unreachable_server_gives_no_display() {
        // Display 9999 does not exist on the test machines.
        let api = xlib().expect("libX11.so.6 is installed on Linux");
        assert_eq!(api.open_display(c":9999"), 0);
    }
}
