//! Explicit constructor families preserve deferred copies of borrowed key/value inputs.
use super::{BaseAllocator, DoubleMapNode, DoubleMapNodeOwner, MapLinkNode};
impl<K: Clone, V> DoubleMapNode<K, V> {
    /// Copies the key after transferring the value; `DoubleMapNode::from_key1_ref(&1, 7, next, next2)`.
    pub fn from_key1_ref(
        key: &K,
        value: V,
        successor: *mut MapLinkNode,
        second: *mut MapLinkNode,
    ) -> Self {
        Self::construct_parts(successor, second, || key.clone(), || value)
    }
    /// Constructs a value before copying its key; `DoubleMapNode::construct_ref(&1, next, next2, || 7)`.
    pub fn construct_ref(
        key: &K,
        successor: *mut MapLinkNode,
        second: *mut MapLinkNode,
        construct: impl FnOnce() -> V,
    ) -> Self {
        Self::construct_parts(successor, second, || key.clone(), construct)
    }
}
impl<K, V: Clone> DoubleMapNode<K, V> {
    /// Copies the value before transferring its key; `DoubleMapNode::from_key2_ref(1, &7, next, next2)`.
    pub fn from_key2_ref(
        key: K,
        value: &V,
        successor: *mut MapLinkNode,
        second: *mut MapLinkNode,
    ) -> Self {
        Self::construct_parts(successor, second, || key, || value.clone())
    }
}
impl<K: Clone, V: Clone> DoubleMapNode<K, V> {
    /// Copies value then key, matching the native base/member order; `DoubleMapNode::from_refs(&1, &7, next, next2)`.
    pub fn from_refs(
        key: &K,
        value: &V,
        successor: *mut MapLinkNode,
        second: *mut MapLinkNode,
    ) -> Self {
        Self::construct_parts(successor, second, || key.clone(), || value.clone())
    }
    /// Allocates before copying value then key; `DoubleMapNode::allocate_ref(&arena, &1, &7, next, next2)`.
    pub fn allocate_ref(
        allocator: &BaseAllocator,
        key: &K,
        value: &V,
        successor: *mut MapLinkNode,
        second: *mut MapLinkNode,
    ) -> DoubleMapNodeOwner<K, V> {
        Self::allocate_with(
            allocator,
            successor,
            second,
            || key.clone(),
            || value.clone(),
        )
    }
}
