//! Narrow connection I/O seam pins owned reset behavior without an X server.
use rustycad_tkernel::tcollection::AsciiString;

pub(super) trait NativeDisplayCalls: Send + Sync {
    fn open(&self, name: &AsciiString) -> usize;
    fn close(&self, display: usize);
    fn free_visual(&self, visual: usize);
    fn delete_atom(&self, display: usize) -> Option<u64>;
}
pub(super) struct XlibDisplayCalls;
#[cfg(target_os = "linux")]
impl NativeDisplayCalls for XlibDisplayCalls {
    fn open(&self, name: &AsciiString) -> usize {
        let Ok(name) = std::ffi::CString::new(name.as_bytes()) else {
            return 0;
        };
        crate::xlib::xlib().map_or(0, |api| api.open_display(&name))
    }
    fn close(&self, display: usize) {
        if let Some(api) = crate::xlib::xlib() {
            api.close_display(display);
        }
    }
    fn free_visual(&self, visual: usize) {
        if let Some(api) = crate::xlib::xlib() {
            api.free(visual);
        }
    }
    fn delete_atom(&self, display: usize) -> Option<u64> {
        crate::xlib::xlib().map(|api| api.intern_atom(display, c"WM_DELETE_WINDOW", false))
    }
}
#[cfg(not(target_os = "linux"))]
impl NativeDisplayCalls for XlibDisplayCalls {
    fn open(&self, _name: &AsciiString) -> usize {
        0
    }
    fn close(&self, _display: usize) {}
    fn free_visual(&self, _visual: usize) {}
    fn delete_atom(&self, _display: usize) -> Option<u64> {
        None
    }
}
