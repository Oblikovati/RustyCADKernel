//! Native DataMapNode inherits the value/link prefix and destroys its key before that base.
#![allow(unsafe_code)]
use super::{AllocatedBox, BaseAllocator, ListValueNode, MapLinkNode};
use std::mem::ManuallyDrop;
mod borrowed;
mod iterator_node;
mod owner;
pub use owner::DataMapNodeOwner;
mod lifetime;

/// A real `NCollection_DataMap::DataMapNode` key/value node; `DataMapNode::new(1, 7, next)`.
#[repr(C)]
pub struct DataMapNode<K, V> {
    // Native base-subobject layout puts the link/value first, but C++ destroys derived members
    // first. ManuallyDrop preserves both layout and key-before-value destruction (OCCT 3d097a).
    value_node: ManuallyDrop<ListValueNode<V>>,
    key: ManuallyDrop<K>,
}
impl<K, V> DataMapNode<K, V> {
    /// Transfers both objects into a node; `DataMapNode::new(1, 7, std::ptr::null_mut())`.
    pub fn new(key: K, value: V, successor: *mut MapLinkNode) -> Self {
        Self::construct_parts(successor, || key, || value)
    }
    /// Constructs the value before transferring its key; `DataMapNode::construct(1, next, || 7)`.
    pub fn construct(key: K, successor: *mut MapLinkNode, construct: impl FnOnce() -> V) -> Self {
        Self::construct_parts(successor, || key, construct)
    }
    fn construct_parts(
        successor: *mut MapLinkNode,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> Self {
        // Keep a normal owner until key construction succeeds so a key panic drops the value.
        let value_node = ListValueNode::construct(successor, construct_value);
        let key = construct_key();
        Self {
            value_node: ManuallyDrop::new(value_node),
            key: ManuallyDrop::new(key),
        }
    }
    /// Borrows the native immutable key; `assert_eq!(node.key(), &1)`.
    pub fn key(&self) -> &K {
        &self.key
    }
    /// Borrows inherited Value; `assert_eq!(node.value(), &7)`.
    pub fn value(&self) -> &V {
        self.value_node.value()
    }
    /// Borrows inherited ChangeValue; `*node.change_value() = 8`.
    pub fn change_value(&mut self) -> &mut V {
        self.value_node.change_value()
    }
    /// Borrows the real inherited ListNode prefix; `node.link().next()`.
    pub fn link(&self) -> &MapLinkNode {
        self.value_node.link()
    }
    /// Edits inherited Next without owning its target; `*node.link_mut().next_mut() = next`.
    pub fn link_mut(&mut self) -> &mut MapLinkNode {
        self.value_node.link_mut()
    }
    /// Allocates before constructing value then key; `DataMapNode::allocate_with(&arena, next, || 1, || 7)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        successor: *mut MapLinkNode,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> DataMapNodeOwner<K, V> {
        let owner = allocator.allocate_value_with(|| {
            Self::construct_parts(successor, construct_key, construct_value)
        });
        DataMapNodeOwner::from_allocated(owner)
    }
    /// Transfers the node to a native callback with its matching allocator; `DataMapNode::into_raw(owner)`.
    pub fn into_raw(owner: DataMapNodeOwner<K, V>) -> (*mut MapLinkNode, BaseAllocator) {
        owner.into_parts()
    }
    /// Destroys key then value and frees only this node; `unsafe { DataMapNode::<i32,i32>::del_node(node, &mut arena) }`.
    /// # Safety
    /// Node must be a live `Self::into_raw` result paired with its allocator and have no live borrows.
    pub unsafe fn del_node(node: *mut MapLinkNode, allocator: &mut BaseAllocator) {
        // SAFETY: repr(C) keeps the real ListValueNode and its ListNode prefix at offset zero.
        unsafe { super::node_release::release_typed_node(node.cast::<Self>(), allocator) };
    }
}
