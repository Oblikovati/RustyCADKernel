//! Real ordered-data nodes retain native value/link/key placement and reverse payload destruction.
#![allow(unsafe_code)]
use super::map_payload::{construct_value_then_key, drop_key_then_value};
use super::ordered_node_links::{OrderedNodeLinks, ordered_node_links};
use super::ordered_node_owner::OrderedNodeOwner;
use super::{BaseAllocator, BasicMapNode, BasicMapNodeMut, MapLinkNode};
use std::mem::ManuallyDrop;
mod borrowed;
/// Allocator-owned ordered-data node; `OrderedDataMapNode::allocate_with(&arena, next, || 1, || 7)`.
pub type OrderedDataMapNodeOwner<K, V> = OrderedNodeOwner<OrderedDataMapNode<K, V>>;
/// Native key/value node with independent hash and insertion-order links; `OrderedDataMapNode::new(1, 7, next)`.
#[repr(C)]
pub struct OrderedDataMapNode<K, V> {
    // OCCT 3d097a03 places order links before the derived key. Flat fields avoid borrowing padded bases.
    pub(super) link: MapLinkNode,
    pub(super) value: ManuallyDrop<V>,
    pub(super) order: OrderedNodeLinks,
    pub(super) key: ManuallyDrop<K>,
}
impl<K, V> OrderedDataMapNode<K, V> {
    /// Transfers key and item while clearing order links; `OrderedDataMapNode::new(1, 7, next)`.
    pub fn new(key: K, value: V, next: *mut MapLinkNode) -> Self {
        Self::construct_parts(next, || key, || value)
    }
    /// Constructs the item before transferring its key; `OrderedDataMapNode::construct(1, next, || 7)`.
    pub fn construct(key: K, next: *mut MapLinkNode, construct: impl FnOnce() -> V) -> Self {
        Self::construct_parts(next, || key, construct)
    }
    fn construct_parts(
        next: *mut MapLinkNode,
        key: impl FnOnce() -> K,
        value: impl FnOnce() -> V,
    ) -> Self {
        let link = MapLinkNode::new(next);
        let (value, key) = construct_value_then_key(value, key);
        Self {
            link,
            value: ManuallyDrop::new(value),
            order: OrderedNodeLinks::default(),
            key: ManuallyDrop::new(key),
        }
    }
    /// Borrows the native immutable key; `assert_eq!(node.key(), &1)`.
    pub fn key(&self) -> &K {
        &self.key
    }
    /// Borrows inherited Value; `assert_eq!(node.value(), &7)`.
    pub fn value(&self) -> &V {
        &self.value
    }
    /// Edits inherited ChangeValue; `*node.change_value() = 8`.
    pub fn change_value(&mut self) -> &mut V {
        &mut self.value
    }
    /// Borrows the real inherited hash-chain prefix; `node.link().next()`.
    pub fn link(&self) -> &MapLinkNode {
        &self.link
    }
    /// Edits inherited Next without following it; `*node.link_mut().next_mut() = next`.
    pub fn link_mut(&mut self) -> &mut MapLinkNode {
        &mut self.link
    }
    /// Allocates before constructing value then key; `OrderedDataMapNode::allocate_with(&arena, next, || 1, || 7)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        next: *mut MapLinkNode,
        key: impl FnOnce() -> K,
        value: impl FnOnce() -> V,
    ) -> OrderedDataMapNodeOwner<K, V> {
        OrderedNodeOwner::from_allocated(
            allocator.allocate_value_with(|| Self::construct_parts(next, key, value)),
        )
    }
    /// Transfers node and allocator to a native deleter; `OrderedDataMapNode::into_raw(owner)`.
    pub fn into_raw(owner: OrderedDataMapNodeOwner<K, V>) -> (*mut MapLinkNode, BaseAllocator) {
        let (pointer, allocator) = owner.into_parts();
        (pointer.cast(), allocator)
    }
    /// Destroys key then value and frees this allocation; `unsafe { OrderedDataMapNode::<i32,i32>::del_node(raw, &mut arena) }`.
    /// # Safety
    /// Node must be an intact into_raw result with its matching allocator and no live borrows.
    pub unsafe fn del_node(node: *mut MapLinkNode, allocator: &mut BaseAllocator) {
        // SAFETY: the flat repr(C) node starts with its actual MapLinkNode prefix.
        unsafe { super::node_release::release_typed_node(node.cast::<Self>(), allocator) };
    }
}
impl<K, V> Drop for OrderedDataMapNode<K, V> {
    fn drop(&mut self) {
        // SAFETY: both distinct payload fields remain live until this single owned destruction.
        unsafe { drop_key_then_value(&mut self.key, &mut self.value) };
    }
}
ordered_node_links!(OrderedDataMapNode<K,V>);
impl<K, V> BasicMapNode for OrderedDataMapNode<K, V> {
    type Key = K;
    type Value = V;
    fn bucket_next(&self) -> *mut Self {
        self.link().next().cast()
    }
    fn order_next(&self) -> *mut Self {
        self.order_next()
    }
    fn map_key(&self) -> &K {
        self.key()
    }
    fn map_value(&self) -> &V {
        self.value()
    }
}
impl<K, V> BasicMapNodeMut for OrderedDataMapNode<K, V> {
    fn map_value_mut(&mut self) -> &mut V {
        self.change_value()
    }
}
