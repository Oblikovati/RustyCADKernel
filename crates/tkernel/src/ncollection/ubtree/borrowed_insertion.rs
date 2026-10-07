//! Borrowed native Add preserves construction requirements without narrowing owned adapters.
use super::{BoundingNode, BoundingVolume, UBTree};
impl<T: Clone + Default, B: BoundingVolume + Default> UBTree<T, B> {
    /// Inserts borrowed inputs in native allocation/copy/default order; duplicates are allowed.
    /// Example: `tree.add_ref(&object, &bounds)`.
    /// Failed unpublished paired construction currently uses Rust cleanup, as `gemmate_ref` does.
    pub fn add_ref(&mut self, object: &T, bounds: &B) -> bool {
        self.insert_leaf_ref(object, bounds);
        true
    }
    pub(in crate::ncollection) fn insert_leaf_ref(&mut self, object: &T, bounds: &B) -> usize {
        let Some(branch) = self.insertion_branch(bounds) else {
            let owner = self
                .resource
                .allocate_value_with(|| BoundingNode::new_ref(object, bounds));
            return self.publish_root(owner);
        };
        let union = self.insertion_union(branch, bounds);
        let inserted = self.gemmate_ref(branch, &union, object, bounds).unwrap();
        // OCCT 3d097a0328 publishes ChangeLastNode before the local union bound is destroyed.
        self.last = Some(inserted);
        inserted
    }
}
