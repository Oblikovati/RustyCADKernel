//! OSD path probe operations (OSD_Path); twin of `tkernel_osd_path_probe.cxx`.
use std::str::SplitWhitespace;

/// The output of a path operation, or `None` when `op` belongs to another family.
pub fn evaluate(_op: &str, _input: &mut SplitWhitespace<'_>) -> Option<String> {
    None
}
