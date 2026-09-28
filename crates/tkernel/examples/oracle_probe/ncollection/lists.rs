//! NCollection lists probe operations (twin of `tkernel_ncollection_lists_probe.cxx`).
use std::str::SplitWhitespace;

/// The output of a lists operation, or `None` when `op` belongs to another family.
pub fn evaluate(_op: &str, _input: &mut SplitWhitespace<'_>) -> Option<String> {
    None
}
