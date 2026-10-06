//! Shared Kill/Clear/owner destruction preserves payload preorder and deferred block release.
use super::{BoundingVolume, UBTree};
use crate::ncollection::destructor_unwind::DeferredDestructorPanic;
impl<T: Clone + Default, B: BoundingVolume> UBTree<T, B> {
    pub(super) fn clear_owned_nodes(&mut self) {
        let root = self.root.take();
        self.last = None;
        let mut unwind = DeferredDestructorPanic::default();
        if let Some(root) = root {
            self.release_subtree(root, &mut unwind);
        }
        self.drain_detached_slots(&mut unwind);
        self.nodes.clear();
        // Retain the original allocator if a Rust callback failed, after complete cleanup.
        unwind.finish(Ok(()));
    }
    fn drain_detached_slots(&mut self, unwind: &mut DeferredDestructorPanic) {
        // Public mutable nodes can detach descendants; the arena still owns those payloads.
        for index in 0..self.nodes.len() {
            if self.nodes[index].is_some() {
                unwind.capture(|| self.release_node(index));
            }
        }
    }
    /// Destroy detached payloads in native preorder without recursive stack growth.
    pub(super) fn release_subtree(&mut self, top: usize, unwind: &mut DeferredDestructorPanic) {
        let mut pending: Vec<usize> = vec![top];
        let mut pairs = Vec::new();
        let mut singles = Vec::new();
        while let Some(index) = pending.pop() {
            let Some(node) = self.node_by_id(index) else {
                continue;
            };
            if let Some([first, second]) = node.children {
                self.nodes.record_child_pairs([first, second], &mut pairs);
                pending.extend([second, first]);
            }
            unwind.capture(|| singles.extend(self.nodes.destroy(index)));
        }
        for pair in pairs {
            self.nodes.release_pair(pair);
        }
        drop(singles);
    }
    pub(super) fn release_node(&mut self, index: usize) {
        drop(self.nodes.destroy(index));
    }
}
