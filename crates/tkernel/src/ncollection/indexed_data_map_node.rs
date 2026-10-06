//! Native indexed data nodes reuse key/value lifetime ordering without retaining base tail padding.
#![allow(unsafe_code)]
use super::{BaseAllocator, MapLinkNode, indexed_node_owner::IndexedNodeAllocation, map_payload};
use std::mem::ManuallyDrop;
mod borrowed;
/// A single native key/value/index node; `IndexedDataMapNode::new(1, 2, 7, next)`.
#[repr(C)]
pub struct IndexedDataMapNode<K, V> {
    // OCCT 3d097a places Key1/Index in inherited TListNode tail padding.
    link: MapLinkNode,
    pub(super) value: ManuallyDrop<V>,
    pub(super) key: ManuallyDrop<K>,
    pub(super) index: i32,
}
/// Allocator-owned indexed data node; `IndexedDataMapNode::allocate_ref(&arena, &1, 2, &7, next)`.
pub type IndexedDataMapNodeOwner<K, V> = IndexedNodeAllocation<IndexedDataMapNode<K, V>>;
impl<K, V> IndexedDataMapNode<K, V> {
    /// Transfers owned key and item into a single node; `IndexedDataMapNode::new(1, -2, 7, next)`.
    pub fn new(key: K, index: i32, value: V, next: *mut MapLinkNode) -> Self {
        Self::construct_parts(index, next, || key, || value)
    }
    /// Constructs the value before storing the index; `IndexedDataMapNode::construct(1, 2, next, || 7)`.
    pub fn construct(
        key: K,
        index: i32,
        next: *mut MapLinkNode,
        construct: impl FnOnce() -> V,
    ) -> Self {
        Self::construct_parts(index, next, || key, construct)
    }
    /// Borrows native mutable Key1; `*node.key1() = 2`.
    pub fn key1(&mut self) -> &mut K {
        &mut self.key
    }
    /// Borrows the key for Rust shared access; `assert_eq!(node.key(), &1)`.
    pub fn key(&self) -> &K {
        &self.key
    }
    /// Borrows the native signed Index field; `*node.index() = -7`.
    pub fn index(&mut self) -> &mut i32 {
        &mut self.index
    }
    /// Reads the stored signed index; `assert_eq!(node.index_value(), -7)`.
    pub fn index_value(&self) -> i32 {
        self.index
    }
    /// Borrows inherited Value; `assert_eq!(node.value(), &7)`.
    pub fn value(&self) -> &V {
        &self.value
    }
    /// Borrows inherited ChangeValue; `*node.change_value() = 8`.
    pub fn change_value(&mut self) -> &mut V {
        &mut self.value
    }
    /// Borrows the actual native ListNode prefix; `node.link().next()`.
    pub fn link(&self) -> &MapLinkNode {
        &self.link
    }
    /// Edits inherited Next without owning its target; `*node.link_mut().next_mut() = next`.
    pub fn link_mut(&mut self) -> &mut MapLinkNode {
        &mut self.link
    }
    /// Allocates before constructing value then key; `IndexedDataMapNode::allocate_with(&arena, 2, next, || 1, || 7)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        index: i32,
        next: *mut MapLinkNode,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> IndexedDataMapNodeOwner<K, V> {
        let owner = allocator.allocate_value_with(|| {
            Self::construct_parts(index, next, construct_key, construct_value)
        });
        IndexedNodeAllocation::from_allocated(owner)
    }
    /// Transfers the original allocation and allocator to a callback; `IndexedDataMapNode::into_raw(owner)`.
    pub fn into_raw(owner: IndexedDataMapNodeOwner<K, V>) -> (*mut MapLinkNode, BaseAllocator) {
        let (pointer, allocator) = owner.into_parts();
        (pointer.cast(), allocator)
    }
    /// Destroys key then value and frees only this node; `unsafe { IndexedDataMapNode::<i32,i32>::del_node(node, &mut arena) }`.
    /// # Safety
    /// Node must be a live Self::into_raw result paired with its allocator and have no outstanding borrows.
    pub unsafe fn del_node(node: *mut MapLinkNode, allocator: &mut BaseAllocator) {
        // SAFETY: the real MapLinkNode prefix remains at offset zero; no padded base is borrowed.
        unsafe { super::node_release::release_typed_node(node.cast::<Self>(), allocator) };
    }
}

impl<K, V> IndexedDataMapNode<K, V> {
    pub(super) fn construct_parts(
        index: i32,
        next: *mut MapLinkNode,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> Self {
        let link = MapLinkNode::new(next);
        let (value, key) = map_payload::construct_value_then_key(construct_value, construct_key);
        Self {
            link,
            value: ManuallyDrop::new(value),
            key: ManuallyDrop::new(key),
            index,
        }
    }
}
impl<K, V> Drop for IndexedDataMapNode<K, V> {
    fn drop(&mut self) {
        // SAFETY: these disjoint fields are initialized and this Drop exclusively owns cleanup.
        unsafe { map_payload::drop_key_then_value(&mut self.key, &mut self.value) };
    }
}
