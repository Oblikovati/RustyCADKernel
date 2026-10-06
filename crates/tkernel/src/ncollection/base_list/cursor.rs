use super::{BaseListStorage, ListLinkNode};
use std::ptr::null_mut;

/// Native BaseList Iterator's public links; `BaseListRawIterator::new(&storage)`.
/// Equality uses only `current`; raw field writes never dereference a node.
#[derive(Clone, Copy, Debug)]
pub struct BaseListRawIterator {
    /// Native `myCurrent`; the caller owns its lifetime and type.
    pub current: *mut ListLinkNode,
    /// Native `myPrevious`; retained even when current is null.
    pub previous: *mut ListLinkNode,
}
impl Default for BaseListRawIterator {
    fn default() -> Self {
        Self {
            current: null_mut(),
            previous: null_mut(),
        }
    }
}
impl BaseListRawIterator {
    /// Starts at the first node, with no predecessor; `BaseListRawIterator::new(&storage)`.
    pub fn new(storage: &BaseListStorage) -> Self {
        Self {
            current: storage.first,
            previous: null_mut(),
        }
    }
    /// Restores native Init's head position; `cursor.init(&storage)`.
    pub fn init(&mut self, storage: &BaseListStorage) {
        *self = Self::new(storage);
    }
    /// Restores native Initialize's head position; `cursor.initialize(&storage)`.
    pub fn initialize(&mut self, storage: &BaseListStorage) {
        self.init(storage);
    }
    /// Tests current without accessing its pointee; `cursor.more()`.
    pub fn more(&self) -> bool {
        !self.current.is_null()
    }
    /// Compares only current pointers, including end positions; `cursor.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.current == other.current
    }
    /// Implements protected init(list, previous), whose null case selects the tail.
    /// Example: `unsafe { cursor.init_after(&storage, previous) }`.
    ///
    /// # Safety
    /// A nonnull predecessor must point to a live readable node.
    pub unsafe fn init_after(&mut self, storage: &BaseListStorage, previous: *mut ListLinkNode) {
        self.previous = previous;
        // SAFETY: the caller guarantees a live predecessor when supplied.
        self.current = if previous.is_null() {
            storage.last
        } else {
            unsafe { (*previous).next() }
        };
    }
}
impl PartialEq for BaseListRawIterator {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl Eq for BaseListRawIterator {}
