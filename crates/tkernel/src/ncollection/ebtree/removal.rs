//! Native Remove phases: kill the sibling branch, rebind the absorbed leaf, update ancestors.
use super::EBTree;
use crate::ncollection::{BoundingVolume, HashKey};
impl<T: Clone + Default + HashKey + PartialEq + std::fmt::Debug, B: BoundingVolume> EBTree<T, B> {
    pub(super) fn remove_indexed_leaf(&mut self, index: usize) {
        let parent = self.tree.node(index).parent_id().unwrap();
        let first = self.tree.node(parent).child_id(0).unwrap();
        let killed = i32::from(first != index);
        let survivor = self.tree.node(parent).child_id(1 - killed).unwrap();
        self.tree.kill(parent, killed);
        if self.tree.node(parent).is_leaf() {
            self.rebind_absorbed_leaf(parent, survivor);
        }
        self.tree.update_ancestors(parent);
    }
    fn rebind_absorbed_leaf(&mut self, parent: usize, survivor: usize) {
        if !self.rebind_leaf(parent) {
            // A stateful Rust hash can miss the key: never leave an index into a released slot.
            self.repair_absorbed_index(parent, survivor);
        }
    }
    fn repair_absorbed_index(&mut self, parent: usize, survivor: usize) {
        for (_, index) in self.objects.items_mut() {
            if *index == survivor {
                *index = parent;
                return;
            }
        }
    }
}
