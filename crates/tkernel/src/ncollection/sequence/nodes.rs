//! Native sequence node prefixes retain non-owning links independently of value ownership.
use super::{AllocatedBox, BaseAllocator};

/// OCCT SeqNode's non-owning bidirectional links; `let node = SeqNode::default()`.
/// Links are never dereferenced by this type. Their pointees may outlive or precede this node.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SeqNode {
    next: *mut Self,
    previous: *mut Self,
}
// SAFETY: links are opaque addresses; safe methods never dereference another thread's memory.
unsafe impl Send for SeqNode {}
// SAFETY: shared access only reads opaque addresses; modifying a link requires an exclusive borrow.
unsafe impl Sync for SeqNode {}
impl Default for SeqNode {
    fn default() -> Self {
        Self {
            next: std::ptr::null_mut(),
            previous: std::ptr::null_mut(),
        }
    }
}
impl SeqNode {
    /// Constructs null predecessor and successor links; `SeqNode::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Returns the non-owning native Next pointer; `assert!(node.next().is_null())`.
    pub fn next(&self) -> *mut Self {
        self.next
    }
    /// Returns the non-owning native Previous pointer; `node.previous()`.
    pub fn previous(&self) -> *mut Self {
        self.previous
    }
    /// Changes only the successor link; `node.set_next(std::ptr::null_mut())`.
    pub fn set_next(&mut self, next: *mut Self) {
        self.next = next;
    }
    /// Changes only the predecessor link; `node.set_previous(std::ptr::null_mut())`.
    pub fn set_previous(&mut self, previous: *mut Self) {
        self.previous = previous;
    }
    /// Allocates a null-linked native node in the selected allocator; `SeqNode::allocate(&arena)`.
    pub fn allocate(allocator: &BaseAllocator) -> AllocatedBox<Self> {
        allocator.allocate_value_with(Self::new)
    }
}

/// Public `Sequence<T>::Node` with a native SeqNode prefix; `SequenceNode::new(7)`.
#[repr(C)]
pub struct SequenceNode<T> {
    pub(super) links: SeqNode,
    pub(super) value: T,
}
impl<T> SequenceNode<T> {
    /// Moves an item into an initially unlinked node; `SequenceNode::new(item)`.
    pub fn new(value: T) -> Self {
        Self {
            links: SeqNode::new(),
            value,
        }
    }
    /// Constructs an item in an initially unlinked node; `SequenceNode::with(|| Item::new())`.
    pub fn with(construct: impl FnOnce() -> T) -> Self {
        Self::new(construct())
    }
    /// Borrows the native base prefix; `node.base().next()`.
    pub fn base(&self) -> &SeqNode {
        &self.links
    }
    /// Mutates a standalone node's base prefix; `node.change_base().set_next(next)`.
    pub fn change_base(&mut self) -> &mut SeqNode {
        &mut self.links
    }
    /// Borrows the contained item; `assert_eq!(node.value(), &7)`.
    pub fn value(&self) -> &T {
        &self.value
    }
    /// Borrows the contained item exclusively; `*node.change_value() = 8`.
    pub fn change_value(&mut self) -> &mut T {
        &mut self.value
    }
    /// Allocates before constructing the item; `SequenceNode::allocate_with(&arena, || item)`.
    pub fn allocate_with(
        allocator: &BaseAllocator,
        construct: impl FnOnce() -> T,
    ) -> AllocatedBox<Self> {
        allocator.allocate_value_with(|| Self::with(construct))
    }
    /// Allocates an already-owned item in the selected allocator; `SequenceNode::allocate(&arena, item)`.
    pub fn allocate(allocator: &BaseAllocator, value: T) -> AllocatedBox<Self> {
        Self::allocate_with(allocator, || value)
    }
    /// Deletes through the retained allocation owner; `SequenceNode::delete(owned_node)`.
    pub fn delete(node: AllocatedBox<Self>) {
        drop(node);
    }
}
impl<T: Clone> Clone for SequenceNode<T> {
    fn clone(&self) -> Self {
        Self {
            links: self.links,
            value: self.value.clone(),
        }
    }
    fn clone_from(&mut self, source: &Self) {
        self.links = source.links;
        self.value.clone_from(&source.value);
    }
}
impl<T: Clone> SequenceNode<T> {
    /// Copies an item into an initially unlinked node; `SequenceNode::from_ref(&item)`.
    pub fn from_ref(value: &T) -> Self {
        Self::new(value.clone())
    }
    /// Allocates before copying an item; `SequenceNode::allocate_ref(&arena, &item)`.
    pub fn allocate_ref(allocator: &BaseAllocator, value: &T) -> AllocatedBox<Self> {
        Self::allocate_with(allocator, || value.clone())
    }
}
impl<T> std::ops::Deref for SequenceNode<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value()
    }
}
impl<T> std::ops::DerefMut for SequenceNode<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.change_value()
    }
}
