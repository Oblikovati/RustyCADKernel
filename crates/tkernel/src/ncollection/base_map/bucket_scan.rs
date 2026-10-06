//! Shared first-nonempty scan; each iterator retains its own end-state semantics.
use super::MapLinkNode;

pub(super) fn scan_bucket_links(
    table: &[*mut MapLinkNode],
    bucket: &mut usize,
) -> *mut MapLinkNode {
    while *bucket < table.len() {
        let node = table[*bucket];
        if !node.is_null() {
            return node;
        }
        *bucket += 1;
    }
    std::ptr::null_mut()
}
