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

//! The perf-meter stopwatch clock: `OSD_Chronometer::GetThreadCPU` user seconds.
use super::CpuClock;
use crate::osd::{CpuTimes, NativeCpuTimes};
/// Native clock with OCCT's platform-specific accounting; `NativeCpuClock.now()`.
#[derive(Clone, Copy, Default)]
pub struct NativeCpuClock;
impl CpuClock for NativeCpuClock {
    fn now(&self) -> f64 {
        // Native Stopwatch::getTime keeps the user part and drops the system part.
        NativeCpuTimes.thread_cpu().0
    }
}
