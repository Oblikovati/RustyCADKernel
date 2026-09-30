//! The narrow platform boundary of the thread family: thread identifiers, Windows thread
//! priority and the online processor count.
#![allow(unsafe_code)]
use std::thread::JoinHandle;

/// `OSD_Thread::Current`: `pthread_self()` on Unix, `GetCurrentThreadId()` on Windows.
#[cfg(unix)]
pub(crate) fn current_thread_id() -> usize {
    // SAFETY: pthread_self has no preconditions and never fails.
    let thread: libc::pthread_t = unsafe { libc::pthread_self() };
    thread as usize
}

/// `OSD_Thread::Current`: `pthread_self()` on Unix, `GetCurrentThreadId()` on Windows.
#[cfg(windows)]
pub(crate) fn current_thread_id() -> usize {
    // SAFETY: GetCurrentThreadId has no preconditions and never fails.
    unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() as usize }
}

/// The identifier `Run` records: the `pthread_t` on Unix, the Win32 thread id on Windows.
#[cfg(unix)]
pub(crate) fn thread_id<T>(handle: &JoinHandle<T>) -> usize {
    use std::os::unix::thread::JoinHandleExt;
    // RawPthread is u64 on Linux and already usize on macOS.
    #[allow(clippy::unnecessary_cast)]
    let thread: usize = handle.as_pthread_t() as usize;
    thread
}

/// The identifier `Run` records: the `pthread_t` on Unix, the Win32 thread id on Windows.
#[cfg(windows)]
pub(crate) fn thread_id<T>(handle: &JoinHandle<T>) -> usize {
    use std::os::windows::io::AsRawHandle;
    // SAFETY: the handle is owned by the live JoinHandle for the duration of the call.
    unsafe { windows_sys::Win32::System::Threading::GetThreadId(handle.as_raw_handle()) as usize }
}

/// `SetThreadPriority` on Windows; the pthread build only stores the priority.
#[cfg(windows)]
pub(crate) fn apply_priority<T>(handle: &JoinHandle<T>, priority: i32) {
    use std::os::windows::io::AsRawHandle;
    // SAFETY: the handle is owned by the live JoinHandle; a refused priority is ignored,
    // as natively.
    unsafe {
        windows_sys::Win32::System::Threading::SetThreadPriority(handle.as_raw_handle(), priority);
    }
}

/// `SetThreadPriority` on Windows; the pthread build only stores the priority.
#[cfg(unix)]
pub(crate) fn apply_priority<T>(_handle: &JoinHandle<T>, _priority: i32) {}

/// `sysconf(_SC_NPROCESSORS_ONLN)`, -1 on error as natively.
#[cfg(unix)]
pub(crate) fn online_processors() -> i32 {
    // SAFETY: sysconf only reads a configuration value.
    let count: libc::c_long = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
    count as i32
}

/// `GetSystemInfo().dwNumberOfProcessors` (the WOW64 `GetNativeSystemInfo` branch reports
/// the same count for a 64-bit Rust process).
#[cfg(windows)]
pub(crate) fn online_processors() -> i32 {
    use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
    // SAFETY: SYSTEM_INFO is plain data for which all-zero bytes are valid.
    let mut info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
    // SAFETY: `info` is a valid, writable SYSTEM_INFO.
    unsafe { GetSystemInfo(&mut info) };
    info.dwNumberOfProcessors as i32
}
