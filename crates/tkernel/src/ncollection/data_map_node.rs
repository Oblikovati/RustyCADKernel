//! Native DataMapNode flattens link/value/key fields so derived keys reuse C++ base tail padding.
#![allow(unsafe_code)]
use super::{AllocatedBox, BaseAllocator, MapLinkNode, map_payload::construct_value_then_key};
use std::mem::ManuallyDrop;
mod borrowed;
mod iterator_node;
mod owner;
pub use owner::DataMapNodeOwner;
mod lifetime;

/// A real `NCollection_DataMap::DataMapNode` key/value node; `DataMapNode::new(1, 7, next)`.
#[repr(C)]
pub struct DataMapNode<K, V> {
    // OCCT 3d097a reuses non-POD base tail padding (int/int key offset12, not16 on x86_64).
    // Separate fields preserve those measured offsets; never borrow a whole ListValueNode base
    // because its Rust padding may overlap the derived key. Payload Drop remains key then value.
    link: MapLinkNode,
    value: ManuallyDrop<V>,
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
    pub(crate) fn construct_parts(
        successor: *mut MapLinkNode,
        construct_key: impl FnOnce() -> K,
        construct_value: impl FnOnce() -> V,
    ) -> Self {
        let link = MapLinkNode::new(successor);
        let (value, key) = construct_value_then_key(construct_value, construct_key);
        Self {
            link,
            value: ManuallyDrop::new(value),
            key: ManuallyDrop::new(key),
        }
    }
    pub(crate) fn key_value_mut(&mut self) -> (&K, &mut V) {
        (&self.key, &mut self.value)
    }
    pub(crate) fn key_mut(&mut self) -> &mut K {
        &mut self.key
    }
    /// Borrows the native immutable key; `assert_eq!(node.key(), &1)`.
    pub fn key(&self) -> &K {
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
        // SAFETY: repr(C) keeps the actual MapLinkNode at offset zero, with this exact typed layout.
        unsafe { super::node_release::release_typed_node(node.cast::<Self>(), allocator) };
    }
}
