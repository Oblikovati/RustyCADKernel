// Created on: 2011-10-05
// Created by: Kirill GAVRILOV
// Copyright (c) 2013-2014 OPEN CASCADE SAS
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

//! Host clocks and memory counters, per platform as in `OSD_Timer`, `OSD_Chronometer`
//! and `OSD_MemInfo`.
use crate::osd::MemInfoCounter;

#[cfg(all(unix, not(target_vendor = "apple")))]
mod linux;
#[cfg(all(unix, not(target_vendor = "apple")))]
pub(super) use linux::{memory_bytes, process_cpu, thread_cpu};

#[cfg(target_vendor = "apple")]
mod apple;
#[cfg(target_vendor = "apple")]
pub(super) use apple::{memory_bytes, process_cpu, thread_cpu};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(super) use windows::{memory_bytes, process_cpu, thread_cpu, wall_clock_time};

/// Counter values in bytes indexed by `MemInfoCounter`; `None` is `size_t(-1)` (unavailable).
pub(super) type Counters = [Option<u64>; 7];

/// Whether `counter` is among the active ones.
pub(super) fn is_active(counters: &[MemInfoCounter], counter: MemInfoCounter) -> bool {
    counters.contains(&counter)
}

/// `OSD_Timer::GetWallClockTime` (non-Windows): `gettimeofday` seconds relative to the
/// whole second of the first call, keeping precision against the 1970 epoch.
#[cfg(not(windows))]
pub(super) fn wall_clock_time() -> f64 {
    use std::sync::OnceLock;
    use std::time::{SystemTime, UNIX_EPOCH};
    static START_SECONDS: OnceLock<u64> = OnceLock::new();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let start = *START_SECONDS.get_or_init(|| now.as_secs());
    (now.as_secs() as i64 - start as i64) as f64 + 0.000001 * f64::from(now.subsec_micros())
}
