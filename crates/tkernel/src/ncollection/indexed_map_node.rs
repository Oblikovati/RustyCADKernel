//! Native protected IndexedMapNode retains an actual value/link prefix and signed index.
#![allow(unsafe_code)]
use super::{BaseAllocator, MapLinkNode, indexed_node_owner::IndexedNodeAllocation};
/// Native key node with its stored signed index; `IndexedMapNode::new(7, 1, next)`.
#[repr(C)]
pub struct IndexedMapNode<K> {
    // OCCT 3d097a reuses TListNode tail padding for Index; nesting a Rust base overpads.
    link: MapLinkNode,
    pub(super) key: K,
    pub(super) index: i32,
}
/// Allocator-owned indexed key node; `IndexedMapNode::allocate_ref(&arena, &7, 1, next)`.
pub type IndexedMapNodeOwner<K> = IndexedNodeAllocation<IndexedMapNode<K>>;
impl<K> IndexedMapNode<K> {
    /// Transfers an owned key and preserves any native int index; `IndexedMapNode::new(7, -1, next)`.
    pub fn new(key: K, index: i32, next: *mut MapLinkNode) -> Self {
        Self {
            link: MapLinkNode::new(next),
            key,
            index,
        }
    }
    /// Constructs the inherited key before the index; `IndexedMapNode::construct(1, next, || 7)`.
    pub fn construct(index: i32, next: *mut MapLinkNode, construct: impl FnOnce() -> K) -> Self {
        Self {
            link: MapLinkNode::new(next),
            key: construct(),
            index,
        }
    }
    /// Borrows native mutable Key1, the same object as inherited Value; `*node.key1() = 8`.
    pub fn key1(&mut self) -> &mut K {
        &mut self.key
    }
    /// Borrows the actual signed native Index field; `*node.index() = -7`.
    pub fn index(&mut self) -> &mut i32 {
        &mut self.index
    }
    /// Reads stored Index without borrowing it exclusively; `assert_eq!(node.index_value(), -7)`.
    pub fn index_value(&self) -> i32 {
        self.index
    }
    /// Borrows inherited Value; `assert_eq!(node.value(), &7)`.
    pub fn value(&self) -> &K {
        &self.key
    }
    /// Borrows inherited ChangeValue; `*node.change_value() = 9`.
    pub fn change_value(&mut self) -> &mut K {
        &mut self.key
    }
    /// Borrows the actual native ListNode prefix; `node.link().next()`.
    pub fn link(&self) -> &MapLinkNode {
        &self.link
    }
    /// Edits Next without owning the target; `*node.link_mut().next_mut() = next`.
    pub fn link_mut(&mut self) -> &mut MapLinkNode {
        &mut self.link
    }
    /// Allocates before key construction; `IndexedMapNode::allocate_with(&arena, 1, next, || 7)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        index: i32,
        next: *mut MapLinkNode,
        construct: impl FnOnce() -> K,
    ) -> IndexedMapNodeOwner<K> {
        IndexedNodeAllocation::from_allocated(
            allocator.allocate_value_with(|| Self::construct(index, next, construct)),
        )
    }
    /// Transfers original node ownership and allocator; `IndexedMapNode::into_raw(owner)`.
    pub fn into_raw(owner: IndexedMapNodeOwner<K>) -> (*mut MapLinkNode, BaseAllocator) {
        let (pointer, allocator) = owner.into_parts();
        (pointer.cast(), allocator)
    }
    /// Destroys only this key node; `unsafe { IndexedMapNode::<i32>::del_node(node, &mut arena) }`.
    /// # Safety
    /// Node must be a live Self::into_raw result with its matching allocator and no outstanding borrows.
    pub unsafe fn del_node(node: *mut MapLinkNode, allocator: &mut BaseAllocator) {
        // SAFETY: repr(C) preserves the actual MapLinkNode prefix; payload fields stay flat.
        unsafe { super::node_release::release_typed_node(node.cast::<Self>(), allocator) };
    }
}
impl<K: Clone> IndexedMapNode<K> {
    /// Copies a borrowed key into the inherited value; `IndexedMapNode::from_ref(&7, 1, next)`.
    pub fn from_ref(key: &K, index: i32, next: *mut MapLinkNode) -> Self {
        Self::construct(index, next, || key.clone())
    }
    /// Allocates node bytes before copying the key; `IndexedMapNode::allocate_ref(&arena, &7, 1, next)`.
    pub fn allocate_ref(
        allocator: &BaseAllocator,
        key: &K,
        index: i32,
        next: *mut MapLinkNode,
    ) -> IndexedMapNodeOwner<K> {
        Self::allocate_with(allocator, index, next, || key.clone())
    }
}
