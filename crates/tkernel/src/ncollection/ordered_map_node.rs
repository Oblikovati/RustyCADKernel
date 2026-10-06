//! Native ordered-set nodes own one key and three independent borrowed links.
#![allow(unsafe_code)]
use super::ordered_node_links::{OrderedNodeLinks, ordered_node_links};
use super::ordered_node_owner::OrderedNodeOwner;
use super::{BaseAllocator, BasicMapNode, ListValueNode, MapLinkNode};
/// Allocator-owned ordered-set node; `OrderedMapNode::allocate_with(&arena, next, || 7)`.
pub type OrderedMapNodeOwner<K> = OrderedNodeOwner<OrderedMapNode<K>>;
/// Real `NCollection_OrderedMap::OrderedMapNode`; `OrderedMapNode::new(7, next)`.
#[repr(C)]
pub struct OrderedMapNode<K> {
    // Native order pointers follow the ListValueNode prefix; pointer alignment prevents tail overlap.
    value_node: ListValueNode<K>,
    order: OrderedNodeLinks,
}
impl<K> OrderedMapNode<K> {
    /// Transfers a key and clears both order links; `OrderedMapNode::new(7, next)`.
    pub fn new(key: K, next: *mut MapLinkNode) -> Self {
        Self::construct(next, || key)
    }
    /// Constructs the inherited key before order links; `OrderedMapNode::construct(next, || 7)`.
    pub fn construct(next: *mut MapLinkNode, construct: impl FnOnce() -> K) -> Self {
        Self {
            value_node: ListValueNode::construct(next, construct),
            order: OrderedNodeLinks::default(),
        }
    }
    /// Borrows Key for either native cv overload; `assert_eq!(node.key(), &7)`.
    pub fn key(&self) -> &K {
        self.value_node.value()
    }
    /// Borrows inherited Value, identical to Key; `assert_eq!(node.value(), node.key())`.
    pub fn value(&self) -> &K {
        self.value_node.value()
    }
    /// Edits inherited ChangeValue; restore linked-map hashing first; `*node.change_value() = 8`.
    pub fn change_value(&mut self) -> &mut K {
        self.value_node.change_value()
    }
    /// Borrows inherited hash-chain storage; `node.link().next()`.
    pub fn link(&self) -> &MapLinkNode {
        self.value_node.link()
    }
    /// Edits inherited hash-chain storage; `*node.link_mut().next_mut() = next`.
    pub fn link_mut(&mut self) -> &mut MapLinkNode {
        self.value_node.link_mut()
    }
    /// Allocates before constructing the key; `OrderedMapNode::allocate_with(&arena, next, || 7)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        next: *mut MapLinkNode,
        construct: impl FnOnce() -> K,
    ) -> OrderedMapNodeOwner<K> {
        OrderedNodeOwner::from_allocated(
            allocator.allocate_value_with(|| Self::construct(next, construct)),
        )
    }
    /// Transfers node and allocator to a native deleter; `OrderedMapNode::into_raw(owner)`.
    pub fn into_raw(owner: OrderedMapNodeOwner<K>) -> (*mut MapLinkNode, BaseAllocator) {
        let (pointer, allocator) = owner.into_parts();
        (pointer.cast(), allocator)
    }
    /// Destroys the key and frees only this node; `unsafe { OrderedMapNode::<i32>::del_node(raw, &mut arena) }`.
    /// # Safety
    /// Node must be an intact into_raw result with its matching allocator and no live borrows.
    pub unsafe fn del_node(node: *mut MapLinkNode, allocator: &mut BaseAllocator) {
        // SAFETY: repr(C) retains the real MapLinkNode prefix at offset zero.
        unsafe { super::node_release::release_typed_node(node.cast::<Self>(), allocator) };
    }
}
impl<K: Clone> OrderedMapNode<K> {
    /// Copies a borrowed key once; `OrderedMapNode::from_ref(&7, next)`.
    pub fn from_ref(key: &K, next: *mut MapLinkNode) -> Self {
        Self::construct(next, || key.clone())
    }
    /// Allocates before copying a key; `OrderedMapNode::allocate_ref(&arena, &7, next)`.
    pub fn allocate_ref(
        allocator: &BaseAllocator,
        key: &K,
        next: *mut MapLinkNode,
    ) -> OrderedMapNodeOwner<K> {
        Self::allocate_with(allocator, next, || key.clone())
    }
}
ordered_node_links!(OrderedMapNode<K>);
impl<K> BasicMapNode for OrderedMapNode<K> {
    type Key = K;
    type Value = K;
    fn bucket_next(&self) -> *mut Self {
        self.link().next().cast()
    }
    fn order_next(&self) -> *mut Self {
        self.order_next()
    }
    fn map_key(&self) -> &K {
        self.key()
    }
    fn map_value(&self) -> &K {
        self.value()
    }
}
// Native mutable set iterators still expose const keys, so BasicMapNodeMut is intentionally absent.
