//! Editable bounding tree maintains object-to-leaf bindings through node moves.
use super::{BaseAllocator, BoundingNode, BoundingVolume, DataMap, HashKey, TreeSelector, UBTree};
use crate::standard::Failure;
/// Unique objects with indexed removal; `EBTree::<i32, Bounds>::default()`.
pub struct EBTree<T: Clone + Default + HashKey + PartialEq + std::fmt::Debug, B: BoundingVolume> {
    tree: UBTree<T, B>,
    objects: DataMap<T, usize>,
}
impl<T: Clone + Default + HashKey + PartialEq + std::fmt::Debug, B: BoundingVolume> Default
    for EBTree<T, B>
{
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}
impl<T: Clone + Default + HashKey + PartialEq + std::fmt::Debug, B: BoundingVolume> EBTree<T, B> {
    /// Inject node allocation; the native object index uses its own default resource.
    /// Example: `EBTree::with_allocator(arena)`.
    pub fn with_allocator(resource: BaseAllocator) -> Self {
        Self {
            tree: UBTree::with_allocator(resource),
            objects: DataMap::default(),
        }
    }
    /// Check for no root; `tree.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
    /// Check the object index; `tree.contains(&1)`.
    pub fn contains(&self, object: &T) -> bool {
        self.objects.is_bound(object)
    }
    /// Reject duplicates without changing bounds; `tree.add(1, bounds)`.
    pub fn add(&mut self, object: T, bounds: B) -> bool {
        if self.contains(&object) {
            return false;
        }
        let index: usize = self.tree.insert_leaf(object.clone(), bounds);
        self.objects.bind(object, index);
        if let Some(previous) = self.tree.previous_leaf(index) {
            self.objects
                .bind(self.tree.node(previous).object().clone(), previous);
        }
        true
    }
    /// Locate a leaf or return Standard_NoSuchObject; `tree.find_node(&1)?`.
    pub fn find_node(&self, object: &T) -> Result<&BoundingNode<T, B>, Failure> {
        Ok(self.tree.node(*self.objects.find(object)?))
    }
    /// Absorb the sibling and update ancestor bounds; `tree.remove(&1)`.
    pub fn remove(&mut self, object: &T) -> bool {
        let Ok(&index) = self.objects.find(object) else {
            return false;
        };
        self.objects.unbind(object);
        if let Some(parent) = self.tree.remove_leaf(index) {
            self.objects
                .bind(self.tree.node(parent).object().clone(), parent);
        }
        true
    }
    /// Select with native branch pruning and stopping; `tree.select(&mut selector)`.
    pub fn select(&self, selector: &mut impl TreeSelector<T, B>) -> i32 {
        self.tree.select(selector)
    }
    /// Release both nodes and object bindings; `tree.clear()`.
    pub fn clear(&mut self) {
        self.clear_with_allocator(None);
    }
    /// Release nodes and bindings, then adopt `allocator` for future nodes when given
    /// (native `Clear(aNewAlloc)`); `tree.clear_with_allocator(Some(arena))`.
    pub fn clear_with_allocator(&mut self, allocator: Option<BaseAllocator>) {
        self.objects.clear(false);
        self.tree.clear_with_allocator(allocator);
    }
    /// The underlying UBTree, for the inherited read-only members (`Root`, `IsEmpty`,
    /// `Allocator`, `Select`); `tree.as_ubtree().root()`.
    pub fn as_ubtree(&self) -> &UBTree<T, B> {
        &self.tree
    }
    /// Transfer tree and index together; `EBTree::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        let replacement: Self = Self::with_allocator(source.tree.allocator().clone());
        std::mem::replace(source, replacement)
    }
    /// Replace old nodes with the source; `tree.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = Self::move_from(source);
    }
}
