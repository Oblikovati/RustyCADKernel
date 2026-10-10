//! Preserve native path-list separators when adding the Tcl pkg-config shim.
use std::ffi::{OsStr, OsString};
use std::path::Path;

/// Prepend the shim without splitting Windows drive letters; `prepend(Path::new("shim"), existing)`.
pub fn prepend(directory: &Path, current: &OsStr) -> Result<OsString, std::env::JoinPathsError> {
    let existing = std::env::split_paths(current).filter(|_| !current.is_empty());
    std::env::join_paths(std::iter::once(directory.to_path_buf()).chain(existing))
}
