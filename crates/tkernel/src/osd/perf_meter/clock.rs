// Created on: 2002-04-03
// Created by: Michael SAZONOV
// Copyright (c) 2002-2014 OPEN CASCADE SAS
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

//! Narrow platform FFI boundary, following OSD_Chronometer::GetThreadCPU.
#![allow(unsafe_code)]
use super::CpuClock;
/// Native clock with OCCT's platform-specific accounting; `NativeCpuClock.now()`.
#[derive(Clone, Copy, Default)]
pub struct NativeCpuClock;
impl CpuClock for NativeCpuClock {
    fn now(&self) -> f64 {
        native_seconds()
    }
}
#[cfg(all(unix, not(target_vendor = "apple")))]
fn native_seconds() -> f64 {
    let mut stamp: libc::timespec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: stamp is initialized, correctly aligned, writable, and alive throughout the call.
    let status: i32 = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut stamp) };
    if status != 0 {
        return 0.0;
    }
    stamp.tv_sec as f64 + 0.000_000_001 * stamp.tv_nsec as f64
}
#[cfg(windows)]
fn native_seconds() -> f64 {
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::Threading::{GetCurrentThread, GetThreadTimes},
    };
    let mut created: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut exited: FILETIME = created;
    let mut kernel: FILETIME = created;
    let mut user: FILETIME = created;
    // SAFETY: current-thread pseudo-handle needs no closing; all four outputs are distinct initialized FILETIMEs.
    let status: i32 = unsafe {
        GetThreadTimes(
            GetCurrentThread(),
            &mut created,
            &mut exited,
            &mut kernel,
            &mut user,
        )
    };
    if status == 0 {
        return 0.0;
    }
    let ticks: u64 = (u64::from(user.dwHighDateTime) << 32) | u64::from(user.dwLowDateTime);
    0.000_000_1 * ticks as f64
}
#[cfg(target_vendor = "apple")]
fn native_seconds() -> f64 {
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
        return 0.0;
    }
    f64::from(info.user_time.seconds) + 0.000_001 * f64::from(info.user_time.microseconds)
}
