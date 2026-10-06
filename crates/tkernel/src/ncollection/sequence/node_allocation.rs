//! Raw callbacks transfer exactly one typed owner while retaining allocator provenance.
use super::{AllocatedBox, BaseAllocator, SeqNode, Sequence, SequenceNode};

impl SeqNode {
    /// Transfers one prefix allocation to a native owner; `SeqNode::into_raw(owner)`.
    pub fn into_raw(owner: AllocatedBox<Self>) -> (*mut Self, BaseAllocator) {
        owner.into_raw_with_allocator()
    }
    /// Reclaims a prefix allocated without a typed item; `unsafe { SeqNode::delete_raw(ptr, &mut arena) }`.
    ///
    /// # Safety
    /// `pointer` must come from SeqNode::into_raw, paired with its returned allocator,
    /// with exclusive ownership and no outstanding references. It must not prefix a typed node.
    pub unsafe fn delete_raw(pointer: *mut Self, allocator: &mut BaseAllocator) {
        // SAFETY: the caller returns exactly the live prefix allocation and its allocator.
        drop(unsafe { AllocatedBox::<Self>::from_raw_in(pointer, allocator.clone()) });
    }
}
impl<T> SequenceNode<T> {
    /// Transfers a typed node through its native base prefix; `SequenceNode::into_raw(owner)`.
    pub fn into_raw(owner: AllocatedBox<Self>) -> (*mut SeqNode, BaseAllocator) {
        let (pointer, allocator) = owner.into_raw_with_allocator();
        (pointer.cast(), allocator)
    }
    /// Destroys only this node's value, then frees its allocation; linked neighbors survive.
    /// Example: `unsafe { SequenceNode::<i32>::del_node(pointer, &mut arena) }`.
    ///
    /// # Safety
    /// `pointer` must come from Self::into_raw for this exact T and its returned allocator,
    /// still owning one live allocation with no outstanding references or earlier deletion.
    pub unsafe fn del_node(pointer: *mut SeqNode, allocator: &mut BaseAllocator) {
        // SAFETY: repr(C) places the prefix first; the caller supplies the original typed owner.
        drop(unsafe { AllocatedBox::<Self>::from_raw_in(pointer.cast(), allocator.clone()) });
    }
}
impl<T> Sequence<T> {
    /// Implements OCCT's static delNode callback for an independently allocated typed node.
    /// Example: `unsafe { Sequence::<i32>::del_node(pointer, &mut arena) }`.
    ///
    /// # Safety
    /// The pointer and allocator must satisfy `SequenceNode::<T>::del_node`'s ownership contract.
    pub unsafe fn del_node(pointer: *mut SeqNode, allocator: &mut BaseAllocator) {
        // SAFETY: this wrapper forwards the caller's exact type and allocation contract.
        unsafe { SequenceNode::<T>::del_node(pointer, allocator) };
    }
}
