// Created on: 1992-11-16
// Created by: Mireille MERCIEN
// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Narrow platform FFI boundary for `OSD_Chronometer::GetProcessCPU` and `GetThreadCPU`.
//! Each function returns (user seconds, system seconds), with the native arithmetic.
#![allow(unsafe_code)]

/// Linux (and every non-Apple Unix): `times()` ticks over `sysconf(_SC_CLK_TCK)`.
#[cfg(all(unix, not(target_vendor = "apple")))]
pub(super) fn process_cpu() -> (f64, f64) {
    tms_seconds(clock_ticks())
}

/// macOS: `times()` ticks. Native code divides by the `CLK_TCK` macro; Rust reads the
/// tick unit from `sysconf(_SC_CLK_TCK)`, the unit `times()` reports on Darwin.
#[cfg(target_vendor = "apple")]
pub(super) fn process_cpu() -> (f64, f64) {
    tms_seconds(clock_ticks())
}

#[cfg(unix)]
fn clock_ticks() -> f64 {
    // SAFETY: sysconf only reads a configuration value.
    let ticks: libc::c_long = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    ticks as f64
}

#[cfg(unix)]
fn tms_seconds(ticks: f64) -> (f64, f64) {
    let mut current: libc::tms = libc::tms {
        tms_utime: 0,
        tms_stime: 0,
        tms_cutime: 0,
        tms_cstime: 0,
    };
    // SAFETY: `current` is initialized, aligned and writable for the whole call; like the
    // native code the (clock_t) return value is ignored.
    unsafe { libc::times(&mut current) };
    (
        current.tms_utime as f64 / ticks,
        current.tms_stime as f64 / ticks,
    )
}

/// Linux: only the user time of a thread is available (`CLOCK_THREAD_CPUTIME_ID`);
/// the system part stays 0, and a failing call leaves both at 0.
#[cfg(all(unix, not(target_vendor = "apple")))]
pub(super) fn thread_cpu() -> (f64, f64) {
    let mut stamp: libc::timespec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: stamp is initialized, correctly aligned, writable, and alive throughout the call.
    let status: i32 = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut stamp) };
    if status != 0 {
        return (0.0, 0.0);
    }
    (
        stamp.tv_sec as f64 + 0.000_000_001 * stamp.tv_nsec as f64,
        0.0,
    )
}

/// macOS: the task's accumulated live-thread times (`TASK_THREAD_TIMES_INFO`), as natively.
#[cfg(target_vendor = "apple")]
pub(super) fn thread_cpu() -> (f64, f64) {
    use mach2::{
        task::task_info,
        task_info::{TASK_THREAD_TIMES_INFO, TASK_THREAD_TIMES_INFO_COUNT, task_thread_times_info},
        traps::mach_task_self,
    };
    let mut info: task_thread_times_info = task_thread_times_info::default();
    let mut count: u32 = TASK_THREAD_TIMES_INFO_COUNT;
    // SAFETY: the flavor/count match the initialized output layout; the task port belongs to this process.
    let status: i32 = unsafe {
        task_info(
            mach_task_self(),
            TASK_THREAD_TIMES_INFO,
            (&mut info as *mut task_thread_times_info).cast(),
            &mut count,
        )
    };
    if status != 0 {
        return (0.0, 0.0);
    }
    (
        f64::from(info.user_time.seconds) + 0.000_001 * f64::from(info.user_time.microseconds),
        f64::from(info.system_time.seconds) + 0.000_001 * f64::from(info.system_time.microseconds),
    )
}

#[cfg(windows)]
fn filetime_seconds(time: windows_sys::Win32::Foundation::FILETIME) -> f64 {
    let ticks: u64 = (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
    // Native EncodeFILETIME yields a signed __int64; FILETIME CPU spans never reach bit 63.
    0.000_000_1 * ticks as i64 as f64
}

#[cfg(windows)]
type TimesQuery = unsafe extern "system" fn(
    windows_sys::Win32::Foundation::HANDLE,
    *mut windows_sys::Win32::Foundation::FILETIME,
    *mut windows_sys::Win32::Foundation::FILETIME,
    *mut windows_sys::Win32::Foundation::FILETIME,
    *mut windows_sys::Win32::Foundation::FILETIME,
) -> windows_sys::core::BOOL;

/// `GetProcessTimes`/`GetThreadTimes` on the current object; like the native code the
/// status is ignored, so a failure reads the zero-initialized times.
#[cfg(windows)]
fn windows_times(query: TimesQuery, object: windows_sys::Win32::Foundation::HANDLE) -> (f64, f64) {
    use windows_sys::Win32::Foundation::FILETIME;
    let zero: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut start, mut exit, mut kernel, mut user) = (zero, zero, zero, zero);
    // SAFETY: the pseudo-handle of the current process/thread needs no closing; all four
    // outputs are distinct initialized FILETIMEs.
    unsafe { query(object, &mut start, &mut exit, &mut kernel, &mut user) };
    (filetime_seconds(user), filetime_seconds(kernel))
}

#[cfg(windows)]
pub(super) fn process_cpu() -> (f64, f64) {
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};
    // SAFETY: GetCurrentProcess returns a constant pseudo-handle.
    windows_times(GetProcessTimes, unsafe { GetCurrentProcess() })
}

#[cfg(windows)]
pub(super) fn thread_cpu() -> (f64, f64) {
    use windows_sys::Win32::System::Threading::{GetCurrentThread, GetThreadTimes};
    // SAFETY: GetCurrentThread returns a constant pseudo-handle.
    windows_times(GetThreadTimes, unsafe { GetCurrentThread() })
}
