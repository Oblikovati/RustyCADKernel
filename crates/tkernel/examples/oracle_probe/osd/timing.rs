//! OSD timing probe operations (OSD_Timer, OSD_Chronometer, OSD_PerfMeter, OSD package statics and signals); twin of `tkernel_osd_timing_probe.cxx`.
use std::str::SplitWhitespace;

/// The output of a timing operation, or `None` when `op` belongs to another family.
pub fn evaluate(_op: &str, _input: &mut SplitWhitespace<'_>) -> Option<String> {
    None
}
