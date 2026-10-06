//! Unordered owners retain allocations independently from canonical BaseList links.
use super::super::node_release::release_typed_node;
use super::{BaseAllocator, ListLinkNode, ListValueNode};
use std::marker::PhantomData;
// Addresses index ownership only; canonical links retain iteration and destruction order.
pub(super) type ListNodeOwners<T> = std::collections::HashMap<usize, ListNodeOwner<T>>;
pub(super) struct ListNodeOwner<T> {
    pub(super) link: *mut ListLinkNode,
    allocator: BaseAllocator,
    item: PhantomData<T>,
}
// SAFETY: item access is mediated by List's exclusive/shared borrows, as for Box<T>.
unsafe impl<T: Send> Send for ListNodeOwner<T> {}
unsafe impl<T: Sync> Sync for ListNodeOwner<T> {}
impl<T> Unpin for ListNodeOwner<T> {}
impl<T> ListNodeOwner<T> {
    pub(super) fn construct(allocator: &BaseAllocator, construct: impl FnOnce() -> T) -> Self {
        let owner = ListValueNode::allocate_with(allocator, std::ptr::null_mut(), construct);
        let (link, allocator) = ListValueNode::into_raw(owner);
        Self {
            link,
            allocator,
            item: PhantomData,
        }
    }
}
impl<T> Drop for ListNodeOwner<T> {
    fn drop(&mut self) {
        // SAFETY: original pointer, allocator and concrete type remain paired in this owner.
        unsafe { release_typed_node(self.link.cast::<ListValueNode<T>>(), &self.allocator) };
    }
}
pub(super) fn take_owner<T>(
    owners: &mut ListNodeOwners<T>,
    node: *mut ListLinkNode,
) -> ListNodeOwner<T> {
    owners.remove(&(node as usize)).unwrap_or_else(|| {
        panic!("native list node {node:p} has no owner; expected exactly one live allocation")
    })
}
pub(super) unsafe fn item_ref<'a, T>(node: *mut ListLinkNode) -> &'a T {
    // SAFETY: caller bounds lifetime by owner and supplies a live shared item.
    unsafe { &*std::ptr::addr_of!((*node.cast::<ListValueNode<T>>()).value) }
}
pub(super) unsafe fn item_mut<'a, T>(node: *mut ListLinkNode) -> &'a mut T {
    // SAFETY: caller retains exclusive item access; no whole-node borrow aliases links.
    unsafe { &mut *std::ptr::addr_of_mut!((*node.cast::<ListValueNode<T>>()).value) }
}

pub(super) fn catch_list_owner_drop<T>(
    owners: &mut ListNodeOwners<T>,
    node: *mut ListLinkNode,
) -> std::thread::Result<()> {
    // Preserve normal callback timing while allowing the raw base to finish after a panic.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        drop(take_owner(owners, node))
    }))
}
