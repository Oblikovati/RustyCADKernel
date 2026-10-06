//! Native Remove phases, with index repair before resuming Rust cleanup panics.
use super::EBTree;
use crate::ncollection::{BoundingVolume, HashKey, destructor_unwind::DeferredDestructorPanic};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
impl<T: Clone + Default + HashKey + PartialEq + std::fmt::Debug, B: BoundingVolume> EBTree<T, B> {
    pub(super) fn remove_indexed_leaf(&mut self, index: usize) {
        let parent = self.tree.node(index).parent_id().unwrap();
        let first = self.tree.node(parent).child_id(0).unwrap();
        let killed = i32::from(first != index);
        let survivor = self.tree.node(parent).child_id(1 - killed).unwrap();
        let mut pending = DeferredDestructorPanic::default();
        pending.capture(|| self.tree.kill(parent, killed));
        if self.tree.node(parent).is_leaf() {
            pending.capture(|| self.rebind_absorbed_leaf(parent, survivor));
        }
        pending.finish(Ok(()));
        self.tree.update_ancestors(parent);
    }
    fn rebind_absorbed_leaf(&mut self, parent: usize, survivor: usize) {
        let result = catch_unwind(AssertUnwindSafe(|| self.rebind_leaf(parent)));
        if !matches!(result, Ok(true)) {
            // A panicking/stateful Rust hash must not leave an index into a released slot.
            self.repair_absorbed_index(parent, survivor);
        }
        if let Err(payload) = result {
            resume_unwind(payload);
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
