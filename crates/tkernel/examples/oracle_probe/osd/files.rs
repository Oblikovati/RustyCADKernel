//! OSD files probe operations (OSD_FileNode, OSD_File, OSD_Directory, OSD_DirectoryIterator, OSD_FileIterator); twin of `tkernel_osd_files_probe.cxx`.
use std::str::SplitWhitespace;

/// The output of a files operation, or `None` when `op` belongs to another family.
pub fn evaluate(_op: &str, _input: &mut SplitWhitespace<'_>) -> Option<String> {
    None
}
