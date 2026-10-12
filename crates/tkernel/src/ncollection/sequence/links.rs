//! Typed node access uses canonical native links; the owner registry carries no item ordering.
use super::{Sequence, SequenceNode};
use crate::standard::Failure;

impl<T> Sequence<T> {
    /// Borrows a checked native node without modifying lookup cache; `sequence.node(1)?.base().next()`.
    pub fn node(&self, index: usize) -> Result<&SequenceNode<T>, Failure> {
        let node = self.checked_node("Node", index, false)?;
        // SAFETY: only shared access while the sequence retains this node allocation.
        Ok(unsafe { &*node.cast::<SequenceNode<T>>() })
    }
}
