//! The real process behind `OSD_Process`: the narrow FFI boundary of the Unix branch
//! (`getuid`, `getpwuid_r`, `localtime_r`) and the standard library elsewhere.
#![allow(unsafe_code)]
use super::{LocalTime, ProcessSource};
use std::io;

/// Queries the running process; `NativeProcess.process_id()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeProcess;

fn os_error(error: io::Error) -> i32 {
    error.raw_os_error().unwrap_or(0)
}

impl ProcessSource for NativeProcess {
    fn process_id(&self) -> i32 {
        std::process::id() as i32
    }
    fn is_super_user(&self) -> bool {
        is_super_user()
    }
    fn user_name(&self) -> Result<Vec<u8>, i32> {
        user_name()
    }
    fn local_time(&self) -> Result<LocalTime, i32> {
        local_time()
    }
    fn current_directory(&self) -> Result<Vec<u8>, i32> {
        std::env::current_dir()
            .map(|path| path_bytes(path.as_os_str()))
            .map_err(os_error)
    }
    fn set_current_directory(&self, path: &[u8]) -> Result<(), i32> {
        std::env::set_current_dir(bytes_path(path)).map_err(os_error)
    }
    fn executable_path(&self) -> Vec<u8> {
        executable_path()
    }
}

#[cfg(unix)]
fn path_bytes(path: &std::ffi::OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    path.as_bytes().to_vec()
}
#[cfg(not(unix))]
fn path_bytes(path: &std::ffi::OsStr) -> Vec<u8> {
    path.to_string_lossy().as_bytes().to_vec()
}
#[cfg(unix)]
fn bytes_path(path: &[u8]) -> std::path::PathBuf {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::OsStr::from_bytes(path).into()
}
#[cfg(not(unix))]
fn bytes_path(path: &[u8]) -> std::path::PathBuf {
    String::from_utf8_lossy(path).into_owned().into()
}

#[cfg(target_os = "linux")]
fn executable_path() -> Vec<u8> {
    // Native: readlink("/proc/<getpid()>/exe"); an unreadable link gives the empty path.
    let link: String = format!("/proc/{}/exe", std::process::id());
    std::fs::read_link(link).map_or_else(|_| Vec::new(), |path| path_bytes(path.as_os_str()))
}
#[cfg(not(target_os = "linux"))]
fn executable_path() -> Vec<u8> {
    // macOS: _NSGetExecutablePath then realpath; Windows: GetModuleFileNameW. The standard
    // library asks the same services; canonicalize is the realpath step.
    let resolved = std::env::current_exe().and_then(|path| {
        if cfg!(target_vendor = "apple") {
            path.canonicalize()
        } else {
            Ok(path)
        }
    });
    resolved.map_or_else(|_| Vec::new(), |path| path_bytes(path.as_os_str()))
}

#[cfg(unix)]
fn is_super_user() -> bool {
    // SAFETY: getuid has no preconditions and cannot fail.
    unsafe { libc::getuid() == 0 }
}
#[cfg(not(unix))]
fn is_super_user() -> bool {
    // Windows membership of the Administrators SID needs the token APIs (pending).
    false
}

#[cfg(unix)]
fn user_name() -> Result<Vec<u8>, i32> {
    let mut entry: libc::passwd = unsafe_zeroed_passwd();
    let mut buffer: Vec<libc::c_char> = vec![0; 16 * 1024];
    let mut found: *mut libc::passwd = std::ptr::null_mut();
    // SAFETY: every pointer refers to live, writable storage of the stated length; the
    // reentrant call writes the entry strings into `buffer` only.
    let status: i32 = unsafe {
        libc::getpwuid_r(
            libc::getuid(),
            &mut entry,
            buffer.as_mut_ptr(),
            buffer.len(),
            &mut found,
        )
    };
    if status != 0 || found.is_null() || entry.pw_name.is_null() {
        return Ok(Vec::new()); // Native: no account entry reads as "".
    }
    // SAFETY: pw_name points into `buffer`, NUL-terminated by getpwuid_r.
    Ok(unsafe { std::ffi::CStr::from_ptr(entry.pw_name) }
        .to_bytes()
        .to_vec())
}
#[cfg(unix)]
fn unsafe_zeroed_passwd() -> libc::passwd {
    // SAFETY: passwd is a plain C struct of integers and pointers; all-zero is valid.
    unsafe { std::mem::zeroed() }
}
#[cfg(not(unix))]
fn user_name() -> Result<Vec<u8>, i32> {
    // Windows GetUserNameW is pending; ERROR_CALL_NOT_IMPLEMENTED is recorded instead.
    Err(120)
}

#[cfg(unix)]
fn local_time() -> Result<LocalTime, i32> {
    let seconds: libc::time_t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as libc::time_t);
    // SAFETY: tm is a plain C struct; localtime_r fills it from a valid time_t.
    let mut broken: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: both pointers refer to live locals for the duration of the call.
    if unsafe { libc::localtime_r(&seconds, &mut broken) }.is_null() {
        return Err(io::Error::last_os_error().raw_os_error().unwrap_or(0));
    }
    Ok(LocalTime {
        month: broken.tm_mon + 1,
        day: broken.tm_mday,
        year: broken.tm_year + 1900,
        hour: broken.tm_hour,
        minute: broken.tm_min,
        second: broken.tm_sec,
        millisecond: 0,
    })
}
#[cfg(not(unix))]
fn local_time() -> Result<LocalTime, i32> {
    // Windows GetLocalTime has no standard-library counterpart (pending).
    Err(120)
}
