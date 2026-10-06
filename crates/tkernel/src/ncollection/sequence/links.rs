//! Typed node access uses canonical native links; the owner registry carries no item ordering.
use super::{SeqNode, Sequence, SequenceNode};
use crate::standard::Failure;

impl<T> Sequence<T> {
    /// Borrows a checked native node without modifying lookup cache; `sequence.node(1)?.base().next()`.
    pub fn node(&self, index: usize) -> Result<&SequenceNode<T>, Failure> {
        let node = self.checked_node("Node", index, false)?;
        // SAFETY: only shared access while the sequence retains this node allocation.
        Ok(unsafe { &*node.cast::<SequenceNode<T>>() })
    }
    pub(super) fn cursor_node(&self, address: Option<usize>) -> Option<(usize, *mut SeqNode)> {
        let address = address?;
        let mut node = self.topology.endpoints().0;
        for offset in 0..self.len() {
            if node as usize == address {
                return Some((offset, node));
            }
            // SAFETY: every native link belongs to this immutable topology.
            node = unsafe { (*node).next() };
        }
        None
    }
}
