//! MapNode is the native key-only specialization of the actual ListValueNode prefix.
#![allow(unsafe_code)]
use super::{BaseAllocator, ListValueNode, MapLinkNode};
mod iterator_node;
mod owner;
pub use owner::MapNodeOwner;
/// A real `NCollection_Map::MapNode`; `MapNode::new(7, std::ptr::null_mut())`.
#[repr(C)]
pub struct MapNode<K> {
    value_node: ListValueNode<K>,
}
impl<K> MapNode<K> {
    /// Transfers an owned key into the inherited value; `MapNode::new(7, next)`.
    pub fn new(key: K, next: *mut MapLinkNode) -> Self {
        Self {
            value_node: ListValueNode::new(key, next),
        }
    }
    /// Constructs the key after establishing its native link; `MapNode::construct(next, || 7)`.
    pub fn construct(next: *mut MapLinkNode, construct: impl FnOnce() -> K) -> Self {
        Self {
            value_node: ListValueNode::construct(next, construct),
        }
    }
    /// Returns the same object as inherited Value for both native Key overloads; `node.key()`.
    pub fn key(&self) -> &K {
        self.value_node.value()
    }
    /// Borrows inherited Value; `assert_eq!(node.value(), node.key())`.
    pub fn value(&self) -> &K {
        self.value_node.value()
    }
    /// Borrows inherited ChangeValue on a standalone node; `*node.change_value()=8`.
    /// A linked map's hash invariants must be restored before its next lookup.
    pub fn change_value(&mut self) -> &mut K {
        self.value_node.change_value()
    }
    /// Borrows the actual native ListNode prefix; `node.link().next()`.
    pub fn link(&self) -> &MapLinkNode {
        self.value_node.link()
    }
    /// Changes inherited Next without owning its target; `*node.link_mut().next_mut()=next`.
    pub fn link_mut(&mut self) -> &mut MapLinkNode {
        self.value_node.link_mut()
    }
    /// Allocates before constructing the key, as native placement new; `MapNode::allocate_with(&arena,next,||7)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        next: *mut MapLinkNode,
        construct: impl FnOnce() -> K,
    ) -> MapNodeOwner<K> {
        MapNodeOwner::from_allocated(
            allocator.allocate_value_with(|| Self::construct(next, construct)),
        )
    }
    /// Transfers node ownership and its allocator to native callbacks; `MapNode::into_raw(owner)`.
    pub fn into_raw(owner: MapNodeOwner<K>) -> (*mut MapLinkNode, BaseAllocator) {
        owner.into_parts()
    }
    /// Destroys the inherited key and frees only this node; `unsafe {MapNode::<i32>::del_node(node,&mut arena)}`.
    /// # Safety
    /// Node must be a live Self::into_raw result paired with its allocator and have no live borrows.
    pub unsafe fn del_node(node: *mut MapLinkNode, allocator: &mut BaseAllocator) {
        // SAFETY: repr(C) keeps the actual ListValueNode and ListNode prefix at offset zero.
        unsafe { super::node_release::release_typed_node(node.cast::<Self>(), allocator) };
    }
}
impl<K: Clone> MapNode<K> {
    /// Copies a borrowed key into inherited Value; `MapNode::from_ref(&7,next)`.
    pub fn from_ref(key: &K, next: *mut MapLinkNode) -> Self {
        Self::construct(next, || key.clone())
    }
    /// Allocates node bytes before copying the borrowed key; `MapNode::allocate_ref(&arena,&7,next)`.
    pub fn allocate_ref(
        allocator: &BaseAllocator,
        key: &K,
        next: *mut MapLinkNode,
    ) -> MapNodeOwner<K> {
        Self::allocate_with(allocator, next, || key.clone())
    }
}
