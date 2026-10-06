//! Owners retain allocation provenance without borrowing complete nodes during link traversal.
use super::super::node_release::release_typed_node;
use super::{BaseAllocator, SeqNode, SequenceNode};
use std::marker::PhantomData;
// Canonical links alone retain order; this index retains original allocation provenance.
pub(super) type SequenceNodeOwners<T> = std::collections::HashMap<usize, SequenceNodeOwner<T>>;

pub(super) struct SequenceNodeOwner<T> {
    pub(super) link: *mut SeqNode,
    allocator: BaseAllocator,
    item: PhantomData<T>,
}
// SAFETY: this owner uniquely owns T; all dereference is mediated by Sequence borrows.
unsafe impl<T: Send> Send for SequenceNodeOwner<T> {}
unsafe impl<T: Sync> Sync for SequenceNodeOwner<T> {}
// Like Box<T>, moving the allocation owner never moves its possibly !Unpin item.
impl<T> Unpin for SequenceNodeOwner<T> {}
impl<T> SequenceNodeOwner<T> {
    pub(super) fn construct(allocator: &BaseAllocator, construct: impl FnOnce() -> T) -> Self {
        let (link, allocator) =
            SequenceNode::into_raw(SequenceNode::allocate_with(allocator, construct));
        Self {
            link,
            allocator,
            item: PhantomData,
        }
    }
}
impl<T> Drop for SequenceNodeOwner<T> {
    fn drop(&mut self) {
        // SAFETY: this owner uniquely retains the original typed node allocation.
        unsafe { release_typed_node(self.link.cast::<SequenceNode<T>>(), &self.allocator) };
    }
}
pub(super) fn take_owner<T>(
    owners: &mut SequenceNodeOwners<T>,
    node: *mut SeqNode,
) -> SequenceNodeOwner<T> {
    owners.remove(&(node as usize)).unwrap_or_else(|| {
        panic!("native sequence node {node:p} has no owner; expected exactly one live allocation")
    })
}
pub(super) unsafe fn item_ref<'a, T>(node: *mut SeqNode) -> &'a T {
    // SAFETY: caller supplies a live T node, shared access and a bounded owner lifetime.
    unsafe { &*std::ptr::addr_of!((*node.cast::<SequenceNode<T>>()).value) }
}
pub(super) unsafe fn item_mut<'a, T>(node: *mut SeqNode) -> &'a mut T {
    // SAFETY: caller supplies exclusive item access without borrowing the node's link prefix.
    unsafe { &mut *std::ptr::addr_of_mut!((*node.cast::<SequenceNode<T>>()).value) }
}
