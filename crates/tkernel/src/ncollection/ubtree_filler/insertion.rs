//! Preserve virtual UBTree::Add when a filler targets an editable derived tree.
use crate::ncollection::{BoundingVolume, EBTree, HashKey, UBTree};
/// Native filler insertion and read-only traversal seam; `UBTreeFiller::new(&mut editable, None, true)`.
pub trait BoundingTreeInsertion<T: Clone + Default, B: BoundingVolume> {
    /// Dispatch native virtual Add; `tree.insert_bound(1,bounds)`.
    fn insert_bound(&mut self, object: T, bounds: B) -> bool;
    /// Borrows queued input before native virtual Add dispatch; `tree.insert_bound_ref(&1, &bounds)`.
    fn insert_bound_ref(&mut self, object: &T, bounds: &B) -> bool {
        self.insert_bound(object.clone(), bounds.clone())
    }
    /// Read the common tree for CheckTree; `tree.tree_view().is_empty()`.
    fn tree_view(&self) -> &UBTree<T, B>;
}
impl<T: Clone + Default, B: BoundingVolume> BoundingTreeInsertion<T, B> for UBTree<T, B> {
    fn insert_bound(&mut self, object: T, bounds: B) -> bool {
        self.add(object, bounds)
    }
    fn tree_view(&self) -> &UBTree<T, B> {
        self
    }
}
impl<T: Clone + Default + HashKey + PartialEq + std::fmt::Debug, B: BoundingVolume>
    BoundingTreeInsertion<T, B> for EBTree<T, B>
{
    fn insert_bound(&mut self, object: T, bounds: B) -> bool {
        self.add(object, bounds)
    }
    fn insert_bound_ref(&mut self, object: &T, bounds: &B) -> bool {
        self.add_ref(object, bounds)
    }
    fn tree_view(&self) -> &UBTree<T, B> {
        self.as_ubtree()
    }
}
