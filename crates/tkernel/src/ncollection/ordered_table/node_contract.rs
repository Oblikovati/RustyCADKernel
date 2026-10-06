//! Concrete native node allocation and field-only projections; no mirrored payloads.
use super::*;
use crate::ncollection::{OrderedDataMapNode, OrderedMapNode};
mod sealed {
    pub trait Sealed {}
}
/// # Safety
/// Implementors own exactly K/V, begin with MapLinkNode, and project initialized disjoint
/// fields. Set V is the immutable unit adaptation and is never mutably projected.
pub(in crate::ncollection) unsafe trait StorageNode<K, V>:
    sealed::Sealed
{
    fn allocate(
        arena: &BaseAllocator,
        next: *mut MapLinkNode,
        key: impl FnOnce() -> K,
        value: impl FnOnce() -> V,
    ) -> *mut MapLinkNode;
    unsafe fn key(node: *mut MapLinkNode) -> *mut K;
    unsafe fn value(node: *mut MapLinkNode) -> *mut V;
    unsafe fn previous(node: *mut MapLinkNode) -> *mut *mut MapLinkNode;
    unsafe fn next(node: *mut MapLinkNode) -> *mut *mut MapLinkNode;
}
impl<K> sealed::Sealed for OrderedMapNode<K> {}
// SAFETY: repr(C) prefix and key/order fields are distinct; unit is only shared by set views.
unsafe impl<K> StorageNode<K, ()> for OrderedMapNode<K> {
    fn allocate(
        arena: &BaseAllocator,
        next: *mut MapLinkNode,
        key: impl FnOnce() -> K,
        value: impl FnOnce(),
    ) -> *mut MapLinkNode {
        Self::into_raw(Self::allocate_with(arena, next, || {
            value();
            key()
        }))
        .0
    }
    unsafe fn key(node: *mut MapLinkNode) -> *mut K {
        unsafe { std::ptr::addr_of_mut!((*node.cast::<Self>()).value_node.value) }
    }
    unsafe fn value(_: *mut MapLinkNode) -> *mut () {
        std::ptr::from_ref(&()).cast_mut()
    }
    unsafe fn previous(node: *mut MapLinkNode) -> *mut *mut MapLinkNode {
        unsafe { std::ptr::addr_of_mut!((*node.cast::<Self>()).order.previous) }
    }
    unsafe fn next(node: *mut MapLinkNode) -> *mut *mut MapLinkNode {
        unsafe { std::ptr::addr_of_mut!((*node.cast::<Self>()).order.next) }
    }
}
impl<K, V> sealed::Sealed for OrderedDataMapNode<K, V> {}
// SAFETY: repr(C) prefix and ManuallyDrop payload fields retain their native addresses.
unsafe impl<K, V> StorageNode<K, V> for OrderedDataMapNode<K, V> {
    fn allocate(
        arena: &BaseAllocator,
        next: *mut MapLinkNode,
        key: impl FnOnce() -> K,
        value: impl FnOnce() -> V,
    ) -> *mut MapLinkNode {
        Self::into_raw(Self::allocate_with(arena, next, key, value)).0
    }
    unsafe fn key(node: *mut MapLinkNode) -> *mut K {
        unsafe { std::ptr::addr_of_mut!((*node.cast::<Self>()).key).cast() }
    }
    unsafe fn value(node: *mut MapLinkNode) -> *mut V {
        unsafe { std::ptr::addr_of_mut!((*node.cast::<Self>()).value).cast() }
    }
    unsafe fn previous(node: *mut MapLinkNode) -> *mut *mut MapLinkNode {
        unsafe { std::ptr::addr_of_mut!((*node.cast::<Self>()).order.previous) }
    }
    unsafe fn next(node: *mut MapLinkNode) -> *mut *mut MapLinkNode {
        unsafe { std::ptr::addr_of_mut!((*node.cast::<Self>()).order.next) }
    }
}
