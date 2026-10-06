//! Native non-owning link prefixes and typed value nodes for custom linked containers.
use super::{AllocatedBox, BaseAllocator, MapLinkNode};

/// The `NCollection_ListNode` shared link prefix; `ListLinkNode::new(std::ptr::null_mut())`.
pub type ListLinkNode = MapLinkNode;

/// `NCollection_TListNode<T>` keeps the non-owning link before its stored value.
/// Use `ListValueNode::new(7, std::ptr::null_mut())` for an unallocated node.
#[repr(C)]
pub struct ListValueNode<T> {
    link: ListLinkNode,
    pub(super) value: T,
}
impl<T> ListValueNode<T> {
    /// Stores an owned value and successor; `ListValueNode::new(7, next)`.
    pub fn new(value: T, successor: *mut ListLinkNode) -> Self {
        Self {
            link: ListLinkNode::new(successor),
            value,
        }
    }
    /// Constructs the value after establishing its link prefix; `ListValueNode::construct(next, || 7)`.
    pub fn construct(successor: *mut ListLinkNode, construct: impl FnOnce() -> T) -> Self {
        Self {
            link: ListLinkNode::new(successor),
            value: construct(),
        }
    }
    /// Borrows the native immutable value; `assert_eq!(node.value(), &7)`.
    pub fn value(&self) -> &T {
        &self.value
    }
    /// Borrows the native mutable value; `*node.change_value() = 8`.
    pub fn change_value(&mut self) -> &mut T {
        &mut self.value
    }
    /// Borrows the shared ListNode prefix; `node.link().next()`.
    pub fn link(&self) -> &ListLinkNode {
        &self.link
    }
    /// Borrows the prefix for native next-link edits; `*node.link_mut().next_mut() = next`.
    pub fn link_mut(&mut self) -> &mut ListLinkNode {
        &mut self.link
    }
    /// Allocates before evaluating the native placement constructor; `ListValueNode::allocate_with(&arena, next, || 7)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        successor: *mut ListLinkNode,
        construct: impl FnOnce() -> T,
    ) -> AllocatedBox<Self> {
        allocator.allocate_value_with(|| Self::construct(successor, construct))
    }
    /// Transfers ownership to a native callback and returns its allocator; `ListValueNode::into_raw(owner)`.
    pub fn into_raw(owner: AllocatedBox<Self>) -> (*mut ListLinkNode, BaseAllocator) {
        let (pointer, allocator) = owner.into_raw_with_allocator();
        (pointer.cast(), allocator)
    }
    /// Destroys only this value and frees its node; successor nodes remain untouched.
    /// Example: `unsafe { ListValueNode::<i32>::del_node(pointer, &mut allocator) }`.
    ///
    /// # Safety
    /// `pointer` must come from `Self::into_raw`, paired with its returned allocator.
    /// It must still own one initialized node of this exact `T`, with no live references.
    pub unsafe fn del_node(pointer: *mut ListLinkNode, allocator: &mut BaseAllocator) {
        // SAFETY: repr(C) places the link first; the caller supplies the exact type and owner.
        let owner = unsafe { AllocatedBox::<Self>::from_raw_in(pointer.cast(), allocator.clone()) };
        drop(owner);
    }
}
impl<T: Clone> ListValueNode<T> {
    /// Copies a borrowed value into a new native node; `ListValueNode::from_ref(&7, next)`.
    pub fn from_ref(value: &T, successor: *mut ListLinkNode) -> Self {
        Self::construct(successor, || value.clone())
    }
    /// Allocates before copying the borrowed value; `ListValueNode::allocate_ref(&arena, &7, next)`.
    pub fn allocate_ref(
        allocator: &BaseAllocator,
        value: &T,
        successor: *mut ListLinkNode,
    ) -> AllocatedBox<Self> {
        Self::allocate_with(allocator, successor, || value.clone())
    }
}
