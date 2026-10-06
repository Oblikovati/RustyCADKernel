//! Native sequence downcasts are explicit so embedded base links need not be the first field.
use super::{SeqNode, SequenceNode};

/// Custom-node operations used by the native sequence BasicIterator template.
/// Example: `unsafe impl BasicSequenceNode for MyNode { /* projection and item access */ }`.
/// # Safety
/// `from_sequence_link` must restore the enclosing node from its own embedded live SeqNode,
/// preserving provenance, alignment and allocation identity, including nonzero field offsets.
pub unsafe trait BasicSequenceNode: Sized {
    /// Item returned by Value and ChangeValue, e.g. `i32`.
    type Item;
    /// Projects an embedded link back to its concrete node; `unsafe { N::from_sequence_link(link) }`.
    /// # Safety
    /// Link must point into a live Self at the exact embedded sequence-link field.
    unsafe fn from_sequence_link(link: *mut SeqNode) -> *mut Self;
    /// Dispatches native Value, including custom node side effects; `node.sequence_value()`.
    fn sequence_value(&self) -> &Self::Item;
}

/// Writable custom-node dispatch, absent on native readonly-only nodes.
/// Example: `impl BasicSequenceNodeMut for MyNode { /* ChangeValue */ }`.
pub trait BasicSequenceNodeMut: BasicSequenceNode {
    /// Dispatches native ChangeValue; `*node.sequence_change_value() = item`.
    fn sequence_change_value(&mut self) -> &mut Self::Item;
}

// SAFETY: SequenceNode is repr(C) and its SeqNode field is first, preserving pointer identity.
unsafe impl<T> BasicSequenceNode for SequenceNode<T> {
    type Item = T;
    unsafe fn from_sequence_link(link: *mut SeqNode) -> *mut Self {
        link.cast()
    }
    fn sequence_value(&self) -> &T {
        self.value()
    }
}
impl<T> BasicSequenceNodeMut for SequenceNode<T> {
    fn sequence_change_value(&mut self) -> &mut T {
        self.change_value()
    }
}
