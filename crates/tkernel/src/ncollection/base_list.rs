//! Type-erased native BaseList storage for externally owned link nodes.
use super::{BaseAllocator, ListLinkNode};
use crate::standard::Failure;
use std::ptr::null_mut;
mod basic_iterator;
mod basic_node;
mod cursor;
mod editing;
mod removal_phase;
mod splicing;
pub use basic_iterator::BaseListBasicIterator;
pub use basic_node::{BasicListNode, BasicListNodeMut};
pub use cursor::BaseListRawIterator;

/// Native node deletion callback; `let deleter: DelListNode = ListValueNode::<i32>::del_node`.
pub type DelListNode = unsafe fn(*mut ListLinkNode, &mut BaseAllocator);

/// Native BaseList fields retained for custom containers; `BaseListStorage::new(None)`.
/// Nodes remain caller-owned: call `clear` with their deleter before dropping the storage.
/// All operations that follow raw links require the caller to keep those nodes valid.
pub struct BaseListStorage {
    /// Native protected allocator slot; custom subclasses use composition in Rust.
    pub allocator: BaseAllocator,
    /// Native protected head; mutating it requires restoring the chain before unsafe edits.
    pub first: *mut ListLinkNode,
    /// Native protected tail; mutating it requires restoring the chain before unsafe edits.
    pub last: *mut ListLinkNode,
    /// Native protected unsigned count; pointer queries remain independent of this value.
    pub length: usize,
}
impl BaseListStorage {
    /// Selects the native common resource for a null handle; `BaseListStorage::new(None)`.
    pub fn new(allocator: Option<BaseAllocator>) -> Self {
        Self {
            allocator: allocator.unwrap_or_else(BaseAllocator::common),
            first: null_mut(),
            last: null_mut(),
            length: 0,
        }
    }
    /// Returns the native signed count; `assert_eq!(storage.extent(), 0)`.
    pub fn extent(&self) -> i32 {
        self.length as i32
    }
    /// Returns the native signed Length alias; `storage.length()`.
    pub fn length(&self) -> i32 {
        self.extent()
    }
    /// Returns native unsigned Size; `storage.size()`.
    pub fn size(&self) -> usize {
        self.length
    }
    /// Tests the first pointer, as native IsEmpty does; `storage.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.first.is_null()
    }
    /// Borrows the native resource identity; `storage.allocator()`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.allocator
    }
    /// Exposes the protected resource slot for custom containers; `*storage.allocator_mut() = arena`.
    pub fn allocator_mut(&mut self) -> &mut BaseAllocator {
        &mut self.allocator
    }
    /// Returns the const first-node pointer without dereferencing it; `storage.first()`.
    pub fn first(&self) -> *const ListLinkNode {
        self.first.cast_const()
    }
    /// Returns native mutable first-node access; `storage.first_mut()`.
    pub fn first_mut(&mut self) -> *mut ListLinkNode {
        self.first
    }
    /// Returns the const last-node pointer without dereferencing it; `storage.last()`.
    pub fn last(&self) -> *const ListLinkNode {
        self.last.cast_const()
    }
    /// Returns native mutable last-node access; `storage.last_mut()`.
    pub fn last_mut(&mut self) -> *mut ListLinkNode {
        self.last
    }
    /// Exchanges every field, including allocator identity; `first.exchange(&mut second)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// Preserves native self-append/prepend's no-op; `storage.splice_self()`.
    pub fn splice_self(&mut self) {}
    fn reset_links(&mut self) {
        self.first = null_mut();
        self.last = null_mut();
        self.length = 0;
    }
}
impl Default for BaseListStorage {
    fn default() -> Self {
        Self::new(None)
    }
}
fn require_position(cursor: &BaseListRawIterator, method: &str) -> Result<(), Failure> {
    if cursor.more() {
        return Ok(());
    }
    Err(Failure::with_type(
        "Standard_NoSuchObject",
        &format!("NCollection_BaseList::{method} on null cursor; expected a current node"),
        "",
    ))
}
