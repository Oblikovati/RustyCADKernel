//! OSD process probe operations (OSD_Process, OSD_Host, OSD_Environment, OSD_SharedLibrary, OSD_MemInfo); twin of `tkernel_osd_process_probe.cxx`.
use std::str::SplitWhitespace;

/// The output of a process operation, or `None` when `op` belongs to another family.
pub fn evaluate(_op: &str, _input: &mut SplitWhitespace<'_>) -> Option<String> {
    None
}
