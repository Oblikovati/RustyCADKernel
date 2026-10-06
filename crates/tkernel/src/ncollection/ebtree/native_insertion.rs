//! Explicit native construction capability preserves the broader existing convenience API.
use super::{BoundingVolume, EBTree, HashKey};
impl<T: Clone + Default + HashKey + PartialEq + std::fmt::Debug, B: BoundingVolume + Default>
    EBTree<T, B>
{
    /// Adds borrowed inputs in native tree/map order, rejecting duplicates before payload copies.
    /// Example: `tree.add_native_ref(&object, &bounds)`.
    /// Payload failures retain native partial publication; unpublished pairs use Rust cleanup.
    /// Hash/equality panics recover in Rust; the native default policy terminates on exceptions.
    pub fn add_native_ref(&mut self, object: &T, bounds: &B) -> bool {
        if self.contains(object) {
            return false;
        }
        let index = self.tree.insert_leaf_ref(object, bounds);
        self.objects.bind_ref(object, &index);
        self.rebind_previous_leaf(index);
        true
    }
}
