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
//! inject deterministic values.
use crate::osd::MemInfoCounter;
mod native;

/// Source of the metric values stored in alert meters; inject a named fake in tests.
pub trait MetricSampler: Send + Sync {
    /// `OSD_Timer::GetWallClockTime`: seconds since the first call; `sampler.wall_clock_time()`.
    fn wall_clock_time(&self) -> f64;
    /// `OSD_Chronometer::GetProcessCPU`: `(user, system)` seconds; `sampler.process_cpu()`.
    fn process_cpu(&self) -> (f64, f64);
    /// `OSD_Chronometer::GetThreadCPU`: `(user, system)` seconds; `sampler.thread_cpu()`.
    fn thread_cpu(&self) -> (f64, f64);
    /// `OSD_MemInfo` with exactly `counters` active, updated once, read as
    /// `ValuePreciseMiB` (-1.0 when unavailable); `sampler.memory_mib(&[MemInfoCounter::Virtual])`.
    fn memory_mib(&self, counters: &[MemInfoCounter]) -> Vec<f64>;
}

/// The host clocks and process memory statistics; `Arc::new(NativeMetricSampler)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeMetricSampler;
impl MetricSampler for NativeMetricSampler {
    fn wall_clock_time(&self) -> f64 {
        native::wall_clock_time()
    }
    fn process_cpu(&self) -> (f64, f64) {
        native::process_cpu()
    }
    fn thread_cpu(&self) -> (f64, f64) {
        native::thread_cpu()
    }
    fn memory_mib(&self, counters: &[MemInfoCounter]) -> Vec<f64> {
        let bytes = native::memory_bytes(counters);
        counters
            .iter()
            .map(|counter| {
                bytes[*counter as usize].map_or(-1.0, |value| value as f64 / (1024.0 * 1024.0))
            })
            .collect()
    }
}
