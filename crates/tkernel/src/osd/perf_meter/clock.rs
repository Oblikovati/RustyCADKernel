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
