//! Thin platform boundary for the native `usleep` and `Sleep` calls.
#![allow(unsafe_code)]

#[cfg(unix)]
pub(super) fn sec_sleep(seconds: i32) {
    let microseconds = seconds.wrapping_mul(1_000).wrapping_mul(1_000) as libc::useconds_t;
    // SAFETY: usleep receives only a scalar duration and touches no caller memory.
    let _ = unsafe { libc::usleep(microseconds) };
}

#[cfg(unix)]
pub(super) fn milli_sec_sleep(milliseconds: i32) {
    let microseconds = milliseconds.wrapping_mul(1_000) as libc::useconds_t;
    // SAFETY: usleep receives only a scalar duration and touches no caller memory.
    let _ = unsafe { libc::usleep(microseconds) };
}

#[cfg(windows)]
pub(super) fn sec_sleep(seconds: i32) {
    let milliseconds = seconds.wrapping_mul(1_000) as u32;
    sleep_windows(milliseconds);
}

#[cfg(windows)]
pub(super) fn milli_sec_sleep(milliseconds: i32) {
    sleep_windows(milliseconds as u32);
}

#[cfg(all(not(unix), not(windows)))]
pub(super) fn sec_sleep(seconds: i32) {
    std::thread::sleep(super::sec_sleep_duration(seconds));
}

#[cfg(all(not(unix), not(windows)))]
pub(super) fn milli_sec_sleep(milliseconds: i32) {
    std::thread::sleep(super::milli_sec_sleep_duration(milliseconds));
}

#[cfg(windows)]
fn sleep_windows(milliseconds: u32) {
    use windows_sys::Win32::System::Threading::Sleep;
    // SAFETY: Sleep accepts a scalar duration and has no pointer parameters.
    unsafe { Sleep(milliseconds) };
}
