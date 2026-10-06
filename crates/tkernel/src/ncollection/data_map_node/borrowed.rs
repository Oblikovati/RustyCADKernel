//! Explicit constructor families preserve deferred copies of borrowed key/value inputs.
use super::{BaseAllocator, DataMapNode, DataMapNodeOwner, MapLinkNode};
impl<K: Clone, V> DataMapNode<K, V> {
    /// Copies the key after transferring the value; `DataMapNode::from_key_ref(&1, 7, next)`.
    pub fn from_key_ref(key: &K, value: V, successor: *mut MapLinkNode) -> Self {
        Self::construct_parts(successor, || key.clone(), || value)
    }
    /// Constructs a value before copying its key; `DataMapNode::construct_ref(&1, next, || 7)`.
    pub fn construct_ref(
        key: &K,
        successor: *mut MapLinkNode,
        construct: impl FnOnce() -> V,
    ) -> Self {
        Self::construct_parts(successor, || key.clone(), construct)
    }
}
impl<K, V: Clone> DataMapNode<K, V> {
    /// Copies the value before transferring its key; `DataMapNode::from_value_ref(1, &7, next)`.
    pub fn from_value_ref(key: K, value: &V, successor: *mut MapLinkNode) -> Self {
        Self::construct_parts(successor, || key, || value.clone())
    }
}
impl<K: Clone, V: Clone> DataMapNode<K, V> {
    /// Copies value then key, matching the native base/member order; `DataMapNode::from_refs(&1, &7, next)`.
    pub fn from_refs(key: &K, value: &V, successor: *mut MapLinkNode) -> Self {
        Self::construct_parts(successor, || key.clone(), || value.clone())
    }
    /// Allocates before copying value then key; `DataMapNode::allocate_ref(&arena, &1, &7, next)`.
    pub fn allocate_ref(
        allocator: &BaseAllocator,
        key: &K,
        value: &V,
        successor: *mut MapLinkNode,
    ) -> DataMapNodeOwner<K, V> {
        Self::allocate_with(allocator, successor, || key.clone(), || value.clone())
    }
}
