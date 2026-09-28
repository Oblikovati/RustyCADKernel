//! OSD file_system probe operations (OSD_FileSystem, OSD_LocalFileSystem, OSD_CachedFileSystem, OSD_FileSystemSelector, OSD_StreamBuffer, OSD_OpenFile); twin of `tkernel_osd_file_system_probe.cxx`.
use std::str::SplitWhitespace;

/// The output of a file_system operation, or `None` when `op` belongs to another family.
pub fn evaluate(_op: &str, _input: &mut SplitWhitespace<'_>) -> Option<String> {
    None
}
