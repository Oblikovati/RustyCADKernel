//! OSD threads probe operations (OSD_Thread, OSD_ThreadPool, OSD_Parallel); twin of `tkernel_osd_threads_probe.cxx`.
use std::str::SplitWhitespace;

/// The output of a threads operation, or `None` when `op` belongs to another family.
pub fn evaluate(_op: &str, _input: &mut SplitWhitespace<'_>) -> Option<String> {
    None
}
