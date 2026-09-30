//! Narrow platform FFI boundary of `Message_PrinterSystemLog`.
#![allow(unsafe_code)]
use crate::message::Gravity;
use crate::tcollection::AsciiString;

#[cfg(unix)]
mod ffi {
    use core::ffi::{c_char, c_int};
    pub(super) const LOG_ERR: c_int = 3;
    pub(super) const LOG_WARNING: c_int = 4;
    pub(super) const LOG_INFO: c_int = 6;
    pub(super) const LOG_DEBUG: c_int = 7;
    pub(super) const LOG_PID: c_int = 0x01;
    pub(super) const LOG_NDELAY: c_int = 0x08;
    pub(super) const LOG_USER: c_int = 1 << 3;
    unsafe extern "C" {
        pub(super) fn openlog(ident: *const c_char, option: c_int, facility: c_int);
        pub(super) fn syslog(priority: c_int, format: *const c_char, ...);
        pub(super) fn closelog();
    }
}

/// `ToCString()`: the bytes up to the first NUL as a C string.
#[cfg(unix)]
fn c_text(text: &AsciiString) -> std::ffi::CString {
    let bytes = text
        .as_bytes()
        .split(|&byte| byte == 0)
        .next()
        .unwrap_or_default();
    std::ffi::CString::new(bytes).expect("bytes cut before the first NUL")
}

/// The opened syslog identity; `openlog` keeps the pointer, so the name lives here.
#[cfg(unix)]
pub(super) struct EventSource {
    _name: std::ffi::CString,
}
#[cfg(unix)]
impl EventSource {
    pub(super) fn open(name: &AsciiString) -> Self {
        let name = c_text(name);
        // SAFETY: `name` outlives the log connection (closed in Drop); flags are valid.
        unsafe { ffi::openlog(name.as_ptr(), ffi::LOG_PID | ffi::LOG_NDELAY, ffi::LOG_USER) };
        Self { _name: name }
    }
    pub(super) fn write(&self, text: &AsciiString, gravity: Gravity) {
        let priority = match gravity {
            Gravity::Trace => ffi::LOG_DEBUG,
            Gravity::Info => ffi::LOG_INFO,
            Gravity::Warning => ffi::LOG_WARNING,
            Gravity::Alarm | Gravity::Fail => ffi::LOG_ERR,
        };
        let message = c_text(text);
        // SAFETY: a literal "%s" format with one valid NUL-terminated string argument.
        unsafe { ffi::syslog(priority, c"%s".as_ptr(), message.as_ptr()) };
    }
}
#[cfg(unix)]
impl Drop for EventSource {
    fn drop(&mut self) {
        // SAFETY: closing the process log connection has no preconditions.
        unsafe { ffi::closelog() };
    }
}

#[cfg(windows)]
mod ffi {
    pub(super) type Handle = *mut core::ffi::c_void;
    pub(super) const EVENTLOG_ERROR_TYPE: u16 = 0x0001;
    pub(super) const EVENTLOG_WARNING_TYPE: u16 = 0x0002;
    pub(super) const EVENTLOG_INFORMATION_TYPE: u16 = 0x0004;
    #[link(name = "advapi32")]
    unsafe extern "system" {
        pub(super) fn RegisterEventSourceW(server: *const u16, source: *const u16) -> Handle;
        pub(super) fn DeregisterEventSource(source: Handle) -> i32;
        pub(super) fn ReportEventW(
            source: Handle,
            kind: u16,
            category: u16,
            event: u32,
            user: *const core::ffi::c_void,
            strings: u16,
            data_size: u32,
            texts: *const *const u16,
            data: *const core::ffi::c_void,
        ) -> i32;
    }
}

/// UTF-16 with terminator of a text decoded as UTF-8 when valid (`TCollection_ExtendedString`).
#[cfg(windows)]
fn wide(text: &AsciiString) -> Vec<u16> {
    let mut units = crate::tcollection::ExtendedString::from_ascii(text)
        .as_utf16()
        .to_vec();
    units.push(0);
    units
}

/// The registered event source; null when registration failed.
#[cfg(windows)]
pub(super) struct EventSource {
    handle: ffi::Handle,
}
// SAFETY: an event-log handle may be used from any thread (ReportEventW is thread-safe).
#[cfg(windows)]
unsafe impl Send for EventSource {}
// SAFETY: see Send; the handle is only read after registration.
#[cfg(windows)]
unsafe impl Sync for EventSource {}
#[cfg(windows)]
impl EventSource {
    pub(super) fn open(name: &AsciiString) -> Self {
        let name = wide(name);
        // SAFETY: null server means the local computer; `name` is NUL-terminated.
        let handle = unsafe { ffi::RegisterEventSourceW(std::ptr::null(), name.as_ptr()) };
        Self { handle }
    }
    pub(super) fn write(&self, text: &AsciiString, gravity: Gravity) {
        if self.handle.is_null() {
            return;
        }
        let kind = match gravity {
            Gravity::Alarm | Gravity::Fail => ffi::EVENTLOG_ERROR_TYPE,
            Gravity::Warning => ffi::EVENTLOG_WARNING_TYPE,
            Gravity::Info | Gravity::Trace => ffi::EVENTLOG_INFORMATION_TYPE,
        };
        let message = wide(text);
        let texts = [message.as_ptr()];
        // SAFETY: one valid NUL-terminated string, no binary data, registered handle.
        unsafe {
            ffi::ReportEventW(
                self.handle,
                kind,
                0,
                0,
                std::ptr::null(),
                1,
                0,
                texts.as_ptr(),
                std::ptr::null(),
            )
        };
    }
}
#[cfg(windows)]
impl Drop for EventSource {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // SAFETY: the handle came from RegisterEventSourceW and is released once.
            unsafe { ffi::DeregisterEventSource(self.handle) };
        }
    }
}
