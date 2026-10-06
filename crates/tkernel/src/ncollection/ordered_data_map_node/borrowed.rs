//! Constructor families defer each borrowed argument copy until its native phase.
use super::{BaseAllocator, MapLinkNode, OrderedDataMapNode, OrderedDataMapNodeOwner};
impl<K: Clone, V> OrderedDataMapNode<K, V> {
    /// Copies the key after transferring the item; `OrderedDataMapNode::from_key_ref(&1, 7, next)`.
    pub fn from_key_ref(key: &K, value: V, next: *mut MapLinkNode) -> Self {
        Self::construct_parts(next, || key.clone(), || value)
    }
    /// Constructs the item before copying the key; `OrderedDataMapNode::construct_ref(&1, next, || 7)`.
    pub fn construct_ref(key: &K, next: *mut MapLinkNode, value: impl FnOnce() -> V) -> Self {
        Self::construct_parts(next, || key.clone(), value)
    }
}
impl<K, V: Clone> OrderedDataMapNode<K, V> {
    /// Copies the item before transferring the key; `OrderedDataMapNode::from_value_ref(1, &7, next)`.
    pub fn from_value_ref(key: K, value: &V, next: *mut MapLinkNode) -> Self {
        Self::construct_parts(next, || key, || value.clone())
    }
}
impl<K: Clone, V: Clone> OrderedDataMapNode<K, V> {
    /// Copies item then key in native base/member order; `OrderedDataMapNode::from_refs(&1, &7, next)`.
    pub fn from_refs(key: &K, value: &V, next: *mut MapLinkNode) -> Self {
        Self::construct_parts(next, || key.clone(), || value.clone())
    }
    /// Allocates before copying item then key; `OrderedDataMapNode::allocate_ref(&arena, &1, &7, next)`.
    pub fn allocate_ref(
        allocator: &BaseAllocator,
        key: &K,
        value: &V,
        next: *mut MapLinkNode,
    ) -> OrderedDataMapNodeOwner<K, V> {
        Self::allocate_with(allocator, next, || key.clone(), || value.clone())
    }
}
