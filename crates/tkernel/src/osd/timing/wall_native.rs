// Created on: 1992-12-04
// Created by: Didier PIFFAULT , Mireille MERCIEN
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

//! Narrow platform FFI boundary for `OSD_Timer::GetWallClockTime`.
#![allow(unsafe_code)]

/// Unix: `gettimeofday` seconds since the first call (the native function-local static
/// keeps `tv_sec` of its first call to avoid precision loss), 0 when the call fails.
#[cfg(unix)]
pub(super) fn wall_clock_seconds() -> f64 {
    use std::sync::OnceLock;
    static START_SECONDS: OnceLock<i64> = OnceLock::new();
    let start: i64 = *START_SECONDS.get_or_init(|| time_of_day().map_or(0, |(sec, _)| sec));
    time_of_day().map_or(0.0, |(sec, usec)| {
        (sec - start) as f64 + 0.000_001 * usec as f64
    })
}

#[cfg(unix)]
// time_t and suseconds_t are narrower than i64 on some Unix targets (macOS tv_usec).
#[allow(clippy::unnecessary_cast)]
fn time_of_day() -> Option<(i64, i64)> {
    let mut time: libc::timeval = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    // SAFETY: `time` is initialized and writable; a null timezone pointer is allowed.
    let status: i32 = unsafe { libc::gettimeofday(&mut time, std::ptr::null_mut()) };
    (status == 0).then_some((time.tv_sec as i64, time.tv_usec as i64))
}

/// Windows: `QueryPerformanceCounter` over the frequency read on the first call, or
/// `0.001 * GetTickCount64()` when the performance counter is unavailable.
#[cfg(windows)]
pub(super) fn wall_clock_seconds() -> f64 {
    use std::sync::OnceLock;
    use windows_sys::Win32::System::{
        Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
        SystemInformation::GetTickCount64,
    };
    static FREQUENCY: OnceLock<Option<i64>> = OnceLock::new();
    let frequency: Option<i64> = *FREQUENCY.get_or_init(|| {
        let mut value: i64 = 0;
        // SAFETY: `value` is a writable i64 alive for the call.
        (unsafe { QueryPerformanceFrequency(&mut value) } != 0).then_some(value)
    });
    let mut counter: i64 = 0;
    // SAFETY: `counter` is a writable i64 alive for the call.
    match frequency {
        Some(frequency) if unsafe { QueryPerformanceCounter(&mut counter) } != 0 => {
            counter as f64 / frequency as f64
        }
        // SAFETY: GetTickCount64 has no preconditions.
        _ => 0.001 * unsafe { GetTickCount64() } as f64,
    }
}
