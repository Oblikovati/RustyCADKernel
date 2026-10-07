//! Native construction capability is selected by a borrowed target, preserving filler bounds.
use super::{BoundingTreeInsertion, BoundingVolume, UBTree};
use crate::ncollection::{EBTree, HashKey};
/// A borrowed tree whose filler insertion uses the native construction domain.
/// Example: `let mut target = tree.native_insertion(); UBTreeFiller::new(&mut target, None, true)`.
/// Queue construction and self-assignment policies are those of the existing filler.
pub struct NativeBoundingTree<'tree, Tree: ?Sized> {
    tree: &'tree mut Tree,
}
impl<T: Clone + Default, B: BoundingVolume + Default> UBTree<T, B> {
    /// Selects native borrowed Add for a filler without narrowing its broader constructor.
    /// Example: `let mut target = tree.native_insertion(); UBTreeFiller::new(&mut target, None, true)`.
    pub fn native_insertion(&mut self) -> NativeBoundingTree<'_, Self> {
        NativeBoundingTree { tree: self }
    }
}
impl<T: Clone + Default + HashKey + PartialEq + std::fmt::Debug, B: BoundingVolume + Default>
    EBTree<T, B>
{
    /// Selects native borrowed Add and editable-tree duplicate dispatch for a filler.
    /// Example: `let mut target = editable.native_insertion(); UBTreeFiller::new(&mut target, None, true)`.
    pub fn native_insertion(&mut self) -> NativeBoundingTree<'_, Self> {
        NativeBoundingTree { tree: self }
    }
}
impl<T: Clone + Default, B: BoundingVolume + Default> BoundingTreeInsertion<T, B>
    for NativeBoundingTree<'_, UBTree<T, B>>
{
    fn insert_bound(&mut self, object: T, bounds: B) -> bool {
        self.insert_bound_ref(&object, &bounds)
    }
    fn insert_bound_ref(&mut self, object: &T, bounds: &B) -> bool {
        self.tree.add_ref(object, bounds)
    }
    fn tree_view(&self) -> &UBTree<T, B> {
        self.tree
    }
}
impl<T: Clone + Default + HashKey + PartialEq + std::fmt::Debug, B: BoundingVolume + Default>
    BoundingTreeInsertion<T, B> for NativeBoundingTree<'_, EBTree<T, B>>
{
    fn insert_bound(&mut self, object: T, bounds: B) -> bool {
        self.insert_bound_ref(&object, &bounds)
    }
    fn insert_bound_ref(&mut self, object: &T, bounds: &B) -> bool {
        self.tree.add_native_ref(object, bounds)
    }
    fn tree_view(&self) -> &UBTree<T, B> {
        self.tree.as_ubtree()
    }
}
