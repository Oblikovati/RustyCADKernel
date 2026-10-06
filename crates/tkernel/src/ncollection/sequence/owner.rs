//! Owners retain allocation provenance without borrowing complete nodes during link traversal.
use super::{BaseAllocator, SeqNode, SequenceNode};
use allocator_api2::alloc::Allocator;
use std::{alloc::Layout, marker::PhantomData, ptr::NonNull};

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
struct SequenceAllocationRelease<'a> {
    allocator: &'a BaseAllocator,
    pointer: NonNull<u8>,
    layout: Layout,
}
impl Drop for SequenceAllocationRelease<'_> {
    fn drop(&mut self) {
        // SAFETY: this guard consumes the original typed allocation exactly once, even on unwind.
        unsafe { self.allocator.deallocate(self.pointer, self.layout) };
    }
}
impl<T> Drop for SequenceNodeOwner<T> {
    fn drop(&mut self) {
        let allocation = SequenceAllocationRelease {
            allocator: &self.allocator,
            pointer: NonNull::new(self.link.cast()).expect("owned sequence node is nonnull"),
            layout: Layout::new::<SequenceNode<T>>(),
        };
        // allocator-api2 0.4.0 Box::drop lacks unwind deallocation. Keep this guard alive
        // while running user destruction, preserving allocator provenance and original layout.
        unsafe { std::ptr::drop_in_place(self.link.cast::<SequenceNode<T>>()) };
        drop(allocation);
    }
}
pub(super) fn take_owner<T>(
    owners: &mut Vec<SequenceNodeOwner<T>>,
    node: *mut SeqNode,
) -> SequenceNodeOwner<T> {
    let index = owners
        .iter()
        .position(|owner| owner.link == node)
        .expect("native sequence node must have one owner");
    owners.swap_remove(index)
}
pub(super) unsafe fn item_ref<'a, T>(node: *mut SeqNode) -> &'a T {
    // SAFETY: caller supplies a live T node, shared access and a bounded owner lifetime.
    unsafe { &*std::ptr::addr_of!((*node.cast::<SequenceNode<T>>()).value) }
}
pub(super) unsafe fn item_mut<'a, T>(node: *mut SeqNode) -> &'a mut T {
    // SAFETY: caller supplies exclusive item access without borrowing the node's link prefix.
    unsafe { &mut *std::ptr::addr_of_mut!((*node.cast::<SequenceNode<T>>()).value) }
}
