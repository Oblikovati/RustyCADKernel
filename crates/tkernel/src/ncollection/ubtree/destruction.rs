//! Shared explicit Kill/Clear destruction without changing implicit owner drop-check bounds.
use super::{BoundingVolume, UBTree};
use crate::ncollection::{destructor_unwind::DeferredDestructorPanic, node_owner::NativeNodeOwner};
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
        while let Some(index) = pending.pop() {
            let Some(node) = self.node_by_id(index) else {
                continue;
            };
            if let Some([first, second]) = node.children {
                pending.extend([second, first]);
            }
            unwind.capture(|| self.release_node(index));
        }
    }
    pub(super) fn release_node(&mut self, index: usize) {
        let owner = self.nodes[index].take().unwrap();
        // The shared owner deallocates even if a payload destructor unwinds.
        drop(NativeNodeOwner::from_allocated(owner));
    }
}
