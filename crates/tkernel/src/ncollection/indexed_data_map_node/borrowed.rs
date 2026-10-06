//! Native constructor overloads copy their inherited value before copying the key.
use super::{BaseAllocator, DataMapNode, IndexedDataMapNode, IndexedDataMapNodeOwner, MapLinkNode};
impl<K: Clone, V> IndexedDataMapNode<K, V> {
    /// Copies the key after transferring the item; `IndexedDataMapNode::from_key_ref(&1, 2, 7, next)`.
    pub fn from_key_ref(key: &K, index: i32, value: V, next: *mut MapLinkNode) -> Self {
        Self {
            value_key_node: DataMapNode::from_key_ref(key, value, next),
            index,
        }
    }
    /// Constructs the value before copying the key; `IndexedDataMapNode::construct_ref(&1, 2, next, || 7)`.
    pub fn construct_ref(
        key: &K,
        index: i32,
        next: *mut MapLinkNode,
        construct: impl FnOnce() -> V,
    ) -> Self {
        Self {
            value_key_node: DataMapNode::construct_ref(key, next, construct),
            index,
        }
    }
}
impl<K, V: Clone> IndexedDataMapNode<K, V> {
    /// Copies the value before transferring its key; `IndexedDataMapNode::from_value_ref(1, 2, &7, next)`.
    pub fn from_value_ref(key: K, index: i32, value: &V, next: *mut MapLinkNode) -> Self {
        Self {
            value_key_node: DataMapNode::from_value_ref(key, value, next),
            index,
        }
    }
}
impl<K: Clone, V: Clone> IndexedDataMapNode<K, V> {
    /// Copies value then key into one node; `IndexedDataMapNode::from_refs(&1, 2, &7, next)`.
    pub fn from_refs(key: &K, index: i32, value: &V, next: *mut MapLinkNode) -> Self {
        Self {
            value_key_node: DataMapNode::from_refs(key, value, next),
            index,
        }
    }
    /// Allocates bytes before copying either object; `IndexedDataMapNode::allocate_ref(&arena, &1, 2, &7, next)`.
    pub fn allocate_ref(
        allocator: &BaseAllocator,
        key: &K,
        index: i32,
        value: &V,
        next: *mut MapLinkNode,
    ) -> IndexedDataMapNodeOwner<K, V> {
        Self::allocate_with(allocator, index, next, || key.clone(), || value.clone())
    }
}
