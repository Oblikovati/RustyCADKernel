//! The narrow FFI boundary of the Unix `OSD_Host`: `uname`, `gethostname`,
//! `gethostbyname`. Other hosts answer empty names.
#![allow(unsafe_code)]
use super::{HostSource, SystemNames};

/// The real machine; `NativeHost.system_names()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeHost;

#[cfg(unix)]
mod unix {
    use super::SystemNames;
    use std::ffi::{CStr, CString};

    // gethostbyname is POSIX (glibc, macOS libSystem) but not bound by the libc crate.
    unsafe extern "C" {
        fn gethostbyname(name: *const libc::c_char) -> *mut libc::hostent;
    }

    fn field(bytes: &[libc::c_char]) -> Vec<u8> {
        // SAFETY: uname NUL-terminates every utsname field inside its array.
        unsafe { CStr::from_ptr(bytes.as_ptr()) }
            .to_bytes()
            .to_vec()
    }

    pub(super) fn system_names() -> SystemNames {
        // SAFETY: utsname is a plain C struct of char arrays; zero is a valid value.
        let mut info: libc::utsname = unsafe { std::mem::zeroed() };
        // SAFETY: `info` is live and writable; the native code ignores the status too.
        unsafe { libc::uname(&mut info) };
        SystemNames {
            sysname: field(&info.sysname),
            release: field(&info.release),
        }
    }

    pub(super) fn host_name() -> Result<Vec<u8>, i32> {
        let mut value: [libc::c_char; 65] = [0; 65];
        // SAFETY: at most 64 bytes are written into the 65-byte buffer, whose last byte
        // stays 0, so the result is always NUL-terminated.
        if unsafe { libc::gethostname(value.as_mut_ptr(), 64) } == -1 {
            return Err(std::io::Error::last_os_error().raw_os_error().unwrap_or(0));
        }
        Ok(field(&value))
    }

    pub(super) fn resolve_ipv4(name: &[u8]) -> Option<[u8; 4]> {
        let name: CString = CString::new(name).ok()?;
        // SAFETY: `name` is a valid C string; the static result is read before any other
        // resolver call on this thread.
        let entry: *mut libc::hostent = unsafe { gethostbyname(name.as_ptr()) };
        if entry.is_null() {
            return None;
        }
        // SAFETY: a non-null hostent from gethostbyname has a null-terminated address list
        // of AF_INET addresses, each h_length (4) bytes long.
        unsafe {
            let first: *mut libc::c_char = *(*entry).h_addr_list;
            if first.is_null() {
                return None;
            }
            let bytes: &[u8] = std::slice::from_raw_parts(first.cast::<u8>(), 4);
            Some([bytes[0], bytes[1], bytes[2], bytes[3]])
        }
    }
}

#[cfg(unix)]
impl HostSource for NativeHost {
    fn system_names(&self) -> SystemNames {
        unix::system_names()
    }
    fn host_name(&self) -> Result<Vec<u8>, i32> {
        unix::host_name()
    }
    fn resolve_ipv4(&self, name: &[u8]) -> Option<[u8; 4]> {
        unix::resolve_ipv4(name)
    }
}

#[cfg(not(unix))]
impl HostSource for NativeHost {
    fn system_names(&self) -> SystemNames {
        SystemNames::default()
    }
    fn host_name(&self) -> Result<Vec<u8>, i32> {
        Ok(Vec::new())
    }
    fn resolve_ipv4(&self, _name: &[u8]) -> Option<[u8; 4]> {
        None
    }
}
