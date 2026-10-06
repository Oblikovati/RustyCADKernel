//! Sealed field projections preserve disjoint payload borrows and account for owner auto traits.
use crate::ncollection::{IndexedDataMapNode, IndexedMapNode};
mod sealed {
    pub trait Sealed {}
}
/// # Safety
/// Each implementor is an allocator-owned concrete node with MapLinkNode at offset zero.
/// Every projection addresses an initialized field; Index is disjoint from payloads.
/// Key and Value alias for set nodes and are disjoint for key/value nodes.
/// Key/Value account for all owned payload traits.
pub(in crate::ncollection) unsafe trait IndexedNode:
    sealed::Sealed
{
    type Key;
    type Value;
    unsafe fn key_ptr(node: *mut Self) -> *mut Self::Key;
    unsafe fn value_ptr(node: *mut Self) -> *mut Self::Value;
    unsafe fn index_ptr(node: *mut Self) -> *mut i32;
}
impl<K> sealed::Sealed for IndexedMapNode<K> {}
// SAFETY: repr(C) flat node owns K and a real link prefix; Value aliases Key for this family.
unsafe impl<K> IndexedNode for IndexedMapNode<K> {
    type Key = K;
    type Value = K;
    unsafe fn key_ptr(node: *mut Self) -> *mut K {
        unsafe { std::ptr::addr_of_mut!((*node).key) }
    }
    unsafe fn value_ptr(node: *mut Self) -> *mut K {
        unsafe { Self::key_ptr(node) }
    }
    unsafe fn index_ptr(node: *mut Self) -> *mut i32 {
        unsafe { std::ptr::addr_of_mut!((*node).index) }
    }
}
impl<K, V> sealed::Sealed for IndexedDataMapNode<K, V> {}
// SAFETY: repr(C) fields key/value are distinct; ManuallyDrop preserves their field layout.
unsafe impl<K, V> IndexedNode for IndexedDataMapNode<K, V> {
    type Key = K;
    type Value = V;
    unsafe fn key_ptr(node: *mut Self) -> *mut K {
        unsafe { std::ptr::addr_of_mut!((*node).key).cast() }
    }
    unsafe fn value_ptr(node: *mut Self) -> *mut V {
        unsafe { std::ptr::addr_of_mut!((*node).value).cast() }
    }
    unsafe fn index_ptr(node: *mut Self) -> *mut i32 {
        unsafe { std::ptr::addr_of_mut!((*node).index) }
    }
}
