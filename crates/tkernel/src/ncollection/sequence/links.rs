//! Keep the public native node prefix coherent with the existing Vec ordering engine.
use super::{SeqNode, Sequence, SequenceNode};
use crate::standard::Failure;

impl<T> Sequence<T> {
    /// Borrows a checked native node without permitting topology mutation; `sequence.node(1)?.base().next()`.
    pub fn node(&self, index: usize) -> Result<&SequenceNode<T>, Failure> {
        let offset = self.offset("Node", index)?;
        Ok(&self.nodes[offset])
    }
    pub(super) fn node_link(&self, offset: usize) -> *mut SeqNode {
        self.nodes.get(offset).map_or(std::ptr::null_mut(), |node| {
            std::ptr::from_ref(node.base()).cast_mut()
        })
    }
    pub(super) fn refresh_node_links(&mut self, offset: usize) {
        let next = self.node_link(offset.saturating_add(1));
        let previous = offset
            .checked_sub(1)
            .map_or(std::ptr::null_mut(), |i| self.node_link(i));
        if let Some(node) = self.nodes.get_mut(offset) {
            node.links.set_next(next);
            node.links.set_previous(previous);
        }
    }
    pub(super) fn refresh_links_around(&mut self, offset: usize) {
        for index in offset.saturating_sub(1)..=offset.saturating_add(1) {
            self.refresh_node_links(index);
        }
    }
    pub(super) fn refresh_all_links(&mut self) {
        for index in 0..self.len() {
            self.refresh_node_links(index);
        }
    }
}
