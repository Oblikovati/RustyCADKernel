//! The process environment behind `OSD_Environment`.
//!
//! Unix keeps the native `putenv` design: each `NAME=value` definition lives in a
//! process-wide buffer (putenv stores the pointer, not a copy), a redefinition replaces
//! the buffer entry and frees the old one only after the new one is installed.
//!
//! Like the native class (and `std::env::set_var` under edition 2024), writing the
//! environment is only sound while no other thread reads or writes it concurrently;
//! callers own that precondition, as they do natively.
#![allow(unsafe_code)]
use super::EnvironmentSource;

/// The real process environment; `NativeEnvironment.get(b"HOME")`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeEnvironment;

#[cfg(unix)]
mod unix {
    use std::ffi::{CStr, CString};
    use std::sync::Mutex;

    // The native `static char** buffer`: definitions handed to putenv, kept alive here.
    static DEFINITIONS: Mutex<Vec<CString>> = Mutex::new(Vec::new());

    pub(super) fn get(name: &[u8]) -> Option<Vec<u8>> {
        let name: CString = CString::new(name).ok()?;
        // SAFETY: `name` is a valid C string; getenv's result is copied before any other
        // environment access by this thread.
        let value: *const libc::c_char = unsafe { libc::getenv(name.as_ptr()) };
        // SAFETY: a non-null getenv result is a NUL-terminated string in the environment.
        (!value.is_null()).then(|| unsafe { CStr::from_ptr(value) }.to_bytes().to_vec())
    }

    pub(super) fn put(name: &[u8], value: &[u8]) -> Result<(), i32> {
        let mut definition: Vec<u8> = name.to_vec();
        definition.push(b'=');
        definition.extend_from_slice(value);
        // Validated names and values hold no NUL; an unvalidated one stops at it, as the
        // native Sprintf("%s=%s") does.
        let definition: CString = CString::new(until_nul(&definition)).expect("NUL removed");
        let mut buffer = DEFINITIONS
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let slot: Option<usize> = buffer.iter().position(|entry| defines(entry, name));
        let pointer: *mut libc::c_char = definition.as_ptr().cast_mut();
        let old: Option<CString> = match slot {
            Some(index) => Some(std::mem::replace(&mut buffer[index], definition)),
            None => {
                buffer.push(definition);
                None
            }
        };
        // SAFETY: `pointer` stays valid while its CString lives in DEFINITIONS; putenv
        // installs it without copying, then the replaced definition may be freed.
        unsafe { libc::putenv(pointer) };
        drop(old);
        get(name).map(|_| ()).ok_or_else(last_errno)
    }

    /// The native test `!strncmp(buffer[i], name, len) && buffer[i][len] == '='`.
    fn defines(entry: &CString, name: &[u8]) -> bool {
        let bytes: &[u8] = entry.as_bytes();
        bytes.len() > name.len() && bytes.starts_with(name) && bytes[name.len()] == b'='
    }

    fn until_nul(bytes: &[u8]) -> &[u8] {
        let end: usize = bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(bytes.len());
        &bytes[..end]
    }

    fn last_errno() -> i32 {
        std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
    }
}

#[cfg(unix)]
impl EnvironmentSource for NativeEnvironment {
    fn get(&self, name: &[u8]) -> Option<Vec<u8>> {
        unix::get(name)
    }
    fn put(&self, name: &[u8], value: &[u8]) -> Result<(), i32> {
        unix::put(name, value)
    }
}

#[cfg(not(unix))]
impl EnvironmentSource for NativeEnvironment {
    fn get(&self, name: &[u8]) -> Option<Vec<u8>> {
        let name: String = String::from_utf8_lossy(name).into_owned();
        std::env::var_os(name).map(|value| value.to_string_lossy().as_bytes().to_vec())
    }
    fn put(&self, name: &[u8], value: &[u8]) -> Result<(), i32> {
        // Native Windows: _wputenv("NAME=value"); an empty value deletes the variable.
        let name: String = String::from_utf8_lossy(name).into_owned();
        let value: String = String::from_utf8_lossy(value).into_owned();
        if name.is_empty() || name.contains(['=', '\0']) || value.contains('\0') {
            return Ok(()); // _wputenv refuses such a definition and OSD ignores its status.
        }
        // SAFETY: the caller guarantees no concurrent environment access (module docs).
        unsafe {
            if value.is_empty() {
                std::env::remove_var(name);
            } else {
                std::env::set_var(name, value);
            }
        }
        Ok(())
    }
}
