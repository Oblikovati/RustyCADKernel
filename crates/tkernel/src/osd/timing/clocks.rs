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

//! The clocks the timing classes read, behind project-owned traits so that tests inject
//! named fakes; the `Native*` implementations are the OCCT platform calls.
use super::{cpu_native, wall_native};

/// CPU-time source of `OSD_Chronometer`; implement for deterministic tests.
pub trait CpuTimes: Send + Sync {
    /// `GetProcessCPU`: (user, system) seconds of the process; `times.process_cpu()`.
    fn process_cpu(&self) -> (f64, f64);
    /// `GetThreadCPU`: (user, system) seconds of the calling thread; `times.thread_cpu()`.
    fn thread_cpu(&self) -> (f64, f64);
}

/// The native CPU clocks: Linux `times()`/`CLOCK_THREAD_CPUTIME_ID` (thread system time
/// is 0), macOS `times()`/`TASK_THREAD_TIMES_INFO`, Windows `GetProcessTimes`/
/// `GetThreadTimes`; `NativeCpuTimes.process_cpu()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeCpuTimes;
impl CpuTimes for NativeCpuTimes {
    fn process_cpu(&self) -> (f64, f64) {
        cpu_native::process_cpu()
    }
    fn thread_cpu(&self) -> (f64, f64) {
        cpu_native::thread_cpu()
    }
}

/// Wall-clock source of `OSD_Timer`; implement for deterministic tests.
pub trait WallClock: Send + Sync {
    /// `GetWallClockTime`: seconds on a monotonic-in-practice scale; `clock.now()`.
    fn now(&self) -> f64;
}

/// The native wall clock: Unix `gettimeofday` relative to the first call's second,
/// Windows `QueryPerformanceCounter`; `NativeWallClock.now()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeWallClock;
impl WallClock for NativeWallClock {
    fn now(&self) -> f64 {
        wall_native::wall_clock_seconds()
    }
}
