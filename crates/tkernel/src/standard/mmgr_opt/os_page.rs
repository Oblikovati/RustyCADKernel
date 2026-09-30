//! Narrow platform FFI boundary for the page size `Standard_MMgrOpt::Initialize` reads.
#![allow(unsafe_code)]

/// The system page size (`getpagesize()` / `GetSystemInfo`); `os_page_size() % 4096 == 0`.
#[cfg(all(unix, not(target_vendor = "apple")))]
pub(super) fn os_page_size() -> usize {
    // SAFETY: sysconf takes a plain integer selector and touches no caller memory.
    let size: libc::c_long = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    usize::try_from(size).unwrap_or(0)
}

/// The system page size (`getpagesize()` / `GetSystemInfo`); `os_page_size() % 4096 == 0`.
#[cfg(target_vendor = "apple")]
pub(super) fn os_page_size() -> usize {
    // SAFETY: vm_page_size is an immutable kernel-initialized global of the Mach runtime.
    unsafe { mach2::vm_page_size::vm_page_size }
}

/// The system page size (`getpagesize()` / `GetSystemInfo`); `os_page_size() % 4096 == 0`.
#[cfg(windows)]
pub(super) fn os_page_size() -> usize {
    use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
    // SAFETY: SYSTEM_INFO is plain data; an all-zero value is a valid initial state.
    let mut info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
    // SAFETY: info is initialized, writable and alive throughout the call.
    unsafe { GetSystemInfo(&mut info) };
    info.dwPageSize as usize
}
