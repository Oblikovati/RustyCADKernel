// Copyright (c) 2020 OPEN CASCADE SAS
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

//! The clock and memory seam behind `Message_AttributeMeter::SetAlertMetrics`.
//!
//! Native code reads `OSD_Timer::GetWallClockTime`, `OSD_Chronometer::GetProcessCPU` /
//! `GetThreadCPU` and `OSD_MemInfo`; a report samples through this trait instead, so tests
//! inject deterministic values. The native sampler delegates to the OSD ports, which own
//! the platform code.
use crate::osd::{CpuTimes, MemCounter, MemInfo, NativeCpuTimes, NativeWallClock, WallClock};

/// Source of the metric values stored in alert meters; inject a named fake in tests.
pub trait MetricSampler: Send + Sync {
    /// `OSD_Timer::GetWallClockTime`: seconds since the first call; `sampler.wall_clock_time()`.
    fn wall_clock_time(&self) -> f64;
    /// `OSD_Chronometer::GetProcessCPU`: `(user, system)` seconds; `sampler.process_cpu()`.
    fn process_cpu(&self) -> (f64, f64);
    /// `OSD_Chronometer::GetThreadCPU`: `(user, system)` seconds; `sampler.thread_cpu()`.
    fn thread_cpu(&self) -> (f64, f64);
    /// `OSD_MemInfo` with exactly `counters` active, updated once, read as
    /// `ValuePreciseMiB` (-1.0 when unavailable); `sampler.memory_mib(&[MemCounter::Virtual])`.
    fn memory_mib(&self, counters: &[MemCounter]) -> Vec<f64>;
}

/// The host clocks and process memory statistics through `osd::NativeWallClock`,
/// `osd::NativeCpuTimes` and `osd::MemInfo`; `Arc::new(NativeMetricSampler)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeMetricSampler;
impl MetricSampler for NativeMetricSampler {
    fn wall_clock_time(&self) -> f64 {
        NativeWallClock.now()
    }
    fn process_cpu(&self) -> (f64, f64) {
        NativeCpuTimes.process_cpu()
    }
    fn thread_cpu(&self) -> (f64, f64) {
        NativeCpuTimes.thread_cpu()
    }
    fn memory_mib(&self, counters: &[MemCounter]) -> Vec<f64> {
        // Message_AttributeMeter builds an inactive OSD_MemInfo, activates the requested
        // counters and updates once.
        let mut info: MemInfo = MemInfo::new(false);
        info.set_active(false);
        for &counter in counters {
            info.set_counter_active(counter, true);
        }
        info.update();
        counters
            .iter()
            .map(|&counter| info.value_precise_mib(counter))
            .collect()
    }
}
