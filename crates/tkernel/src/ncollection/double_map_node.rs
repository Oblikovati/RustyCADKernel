//! DoubleMapNode owns two keys and two non-owning native hash-chain links.
#![allow(unsafe_code)]
use super::{AllocatedBox, BaseAllocator, MapLinkNode, map_payload::construct_value_then_key};
use std::mem::ManuallyDrop;
mod borrowed;
mod iterator_node;
mod owner;
pub use owner::DoubleMapNodeOwner;
mod lifetime;
mod links;

/// A real dual-key node with two borrowed chain links; `DoubleMapNode::new(1, 7, next, next2)`.
#[repr(C)]
pub struct DoubleMapNode<K, V> {
    // OCCT 3d097a reuses non-POD base tail padding (int/int key offset12, not16 on x86_64).
    // Separate fields preserve those measured offsets; never borrow a whole ListValueNode base
    // because its Rust padding may overlap the derived key. Payload Drop remains key then value.
    link: MapLinkNode,
    value: ManuallyDrop<V>,
    key: ManuallyDrop<K>,
    second_next: *mut Self,
}
impl<K, V> DoubleMapNode<K, V> {
    /// Transfers both objects into a node; `DoubleMapNode::new(1, 7, std::ptr::null_mut(), std::ptr::null_mut())`.
    pub fn new(key: K, value: V, successor: *mut MapLinkNode, second: *mut MapLinkNode) -> Self {
        Self::construct_parts(successor, second, || key, || value)
    }
    /// Constructs the value before transferring its key; `DoubleMapNode::construct(1, next, next2, || 7)`.
    pub fn construct(
        key: K,
        successor: *mut MapLinkNode,
        second: *mut MapLinkNode,
        construct: impl FnOnce() -> V,
    ) -> Self {
        Self::construct_parts(successor, second, || key, construct)
    }
    fn construct_parts(
        successor: *mut MapLinkNode,
        second: *mut MapLinkNode,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> Self {
        let link = MapLinkNode::new(successor);
        let (value, key) = construct_value_then_key(construct_value, construct_key);
        Self {
            link,
            value: ManuallyDrop::new(value),
            key: ManuallyDrop::new(key),
            second_next: second.cast(),
        }
    }
    /// Borrows the native immutable key; `assert_eq!(node.key1(), &1)`.
    pub fn key1(&self) -> &K {
        &self.key
    }
    /// Borrows inherited Value; `assert_eq!(node.value(), &7)`.
    pub fn value(&self) -> &V {
        &self.value
    }
    /// Borrows inherited ChangeValue; `*node.change_value() = 8`.
    pub fn change_value(&mut self) -> &mut V {
        &mut self.value
    }
    /// Borrows the real inherited ListNode prefix; `node.link().next()`.
    pub fn link(&self) -> &MapLinkNode {
        &self.link
    }
    /// Edits inherited Next without owning its target; `*node.link_mut().next_mut() = next`.
    pub fn link_mut(&mut self) -> &mut MapLinkNode {
        &mut self.link
    }
    /// Allocates before constructing value then key; `DoubleMapNode::allocate_with(&arena, next, next2, || 1, || 7)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        successor: *mut MapLinkNode,
        second: *mut MapLinkNode,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> DoubleMapNodeOwner<K, V> {
        let owner = allocator.allocate_value_with(|| {
            Self::construct_parts(successor, second, construct_key, construct_value)
        });
        DoubleMapNodeOwner::from_allocated(owner)
    }
    /// Transfers the node to a native callback with its matching allocator; `DoubleMapNode::into_raw(owner)`.
    pub fn into_raw(owner: DoubleMapNodeOwner<K, V>) -> (*mut MapLinkNode, BaseAllocator) {
        owner.into_parts()
    }
    /// Destroys key then value and frees only this node; `unsafe { DoubleMapNode::<i32,i32>::del_node(node, &mut arena) }`.
    /// # Safety
    /// Node must be a live `Self::into_raw` result paired with its allocator and have no live borrows.
    pub unsafe fn del_node(node: *mut MapLinkNode, allocator: &mut BaseAllocator) {
        // SAFETY: repr(C) keeps the actual MapLinkNode at offset zero, with this exact typed layout.
        unsafe { super::node_release::release_typed_node(node.cast::<Self>(), allocator) };
    }
}
