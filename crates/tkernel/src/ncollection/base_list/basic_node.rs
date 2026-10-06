//! Custom node projection preserves native downcasts even with an offset link base.
use super::ListLinkNode;
use crate::ncollection::ListValueNode;

/// Read operations required by native const list iterators.
/// Example: `unsafe impl BasicListNode for MyNode { /* projection and Value */ }`.
///
/// # Safety
/// `from_list_link` must recover the enclosing live node with its original provenance,
/// alignment and allocation identity, including any nonzero embedded-link offset.
pub unsafe trait BasicListNode: Sized {
    /// Native iterator value type, e.g. `i32`.
    type Item;
    /// Recovers the enclosing node; `unsafe { N::from_list_link(link) }`.
    ///
    /// # Safety
    /// `link` must point to this exact node type's live embedded ListLinkNode and
    /// carry access provenance for the entire node, not only a borrowed link field.
    unsafe fn from_list_link(link: *mut ListLinkNode) -> *mut Self;
    /// Calls native const Value, preserving custom dispatch; `node.list_value()`.
    fn list_value(&self) -> &Self::Item;
}

/// Additional dispatch required only by native writable list iterators.
/// Example: `impl BasicListNodeMut for MyNode { /* ChangeValue */ }`.
pub trait BasicListNodeMut: BasicListNode {
    /// Calls native ChangeValue; `*node.list_change_value() = item`.
    fn list_change_value(&mut self) -> &mut Self::Item;
}

// SAFETY: repr(C) ListValueNode stores its ListLinkNode first.
unsafe impl<T> BasicListNode for ListValueNode<T> {
    type Item = T;
    unsafe fn from_list_link(link: *mut ListLinkNode) -> *mut Self {
        link.cast()
    }
    fn list_value(&self) -> &T {
        self.value()
    }
}
impl<T> BasicListNodeMut for ListValueNode<T> {
    fn list_change_value(&mut self) -> &mut T {
        self.change_value()
    }
}
