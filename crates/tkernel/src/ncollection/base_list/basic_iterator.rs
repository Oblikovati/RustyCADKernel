//! Detached native cursors require explicit lifetime and alias guarantees at dereference.
use super::{BaseListRawIterator, BaseListStorage, BasicListNode};
use std::marker::PhantomData;
mod access;

/// Native BaseList BasicIterator over arbitrary embedded list nodes.
/// Example: `BaseListBasicIterator::<ListValueNode<i32>>::new(&storage)`.
/// `MUTABLE=true` enables writable native access; copied cursors do not own nodes.
pub struct BaseListBasicIterator<N: BasicListNode, const MUTABLE: bool = false> {
    /// Native inherited iterator fields; `cursor.raw.current`.
    pub raw: BaseListRawIterator,
    node_type: PhantomData<fn() -> N>,
}
impl<N: BasicListNode, const M: bool> Copy for BaseListBasicIterator<N, M> {}
impl<N: BasicListNode, const M: bool> Clone for BaseListBasicIterator<N, M> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<N: BasicListNode, const M: bool> Default for BaseListBasicIterator<N, M> {
    fn default() -> Self {
        Self::from_raw(BaseListRawIterator::default())
    }
}
impl<N: BasicListNode, const M: bool> BaseListBasicIterator<N, M> {
    /// Wraps the native inherited public state; `Self::from_raw(raw)`.
    pub fn from_raw(raw: BaseListRawIterator) -> Self {
        Self {
            raw,
            node_type: PhantomData,
        }
    }
    /// Copies same-specialization state and returns the destination; `cursor.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        self.raw = other.raw;
        self
    }
    /// Preserves state during native self assignment; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Checks the current pointer without reading its target; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.raw.more()
    }
    /// Returns the native Iterator view of this cursor; `cursor.iterator()`.
    pub fn iterator(&self) -> &Self {
        self
    }
    /// Borrows the native ChangeIterator view; `cursor.change_iterator()`.
    pub fn change_iterator(&mut self) -> &mut Self {
        self
    }
    /// Compares current pointers across const specializations; `cursor.is_equal(&other)`.
    pub fn is_equal<const O: bool>(&self, other: &BaseListBasicIterator<N, O>) -> bool {
        self.raw == other.raw
    }
    /// Performs native prefix increment; exhausted cursors preserve previous.
    /// Example: `unsafe { cursor.prefix_increment() }`.
    ///
    /// # Safety
    /// A nonnull current pointer must address a live readable ListLinkNode.
    pub unsafe fn prefix_increment(&mut self) -> &mut Self {
        if self.more() {
            self.raw.previous = self.raw.current;
            // SAFETY: the caller guarantees readable native link storage.
            self.raw.current = unsafe { (*self.raw.current).next() };
        }
        self
    }
    /// Returns the old native position and advances; `unsafe { cursor.postfix_increment() }`.
    ///
    /// # Safety
    /// The current link must satisfy prefix_increment's live-node contract.
    pub unsafe fn postfix_increment(&mut self) -> Self {
        let old = *self;
        // SAFETY: caller supplies prefix increment's readable-link contract.
        unsafe { self.prefix_increment() };
        old
    }
    /// Uses native Next's prefix increment behavior; `unsafe { cursor.next() }`.
    ///
    /// # Safety
    /// The current link must satisfy prefix_increment's live-node contract.
    pub unsafe fn next(&mut self) {
        // SAFETY: caller guarantees the current link remains readable.
        unsafe { self.prefix_increment() };
    }
}
macro_rules! initialization {
    ($mutable:literal, $borrow:ty) => {
        impl<N: BasicListNode> BaseListBasicIterator<N, $mutable> {
            /// Starts at the native head; `Self::new(storage)`.
            pub fn new(storage: $borrow) -> Self {
                Self::from_raw(BaseListRawIterator::new(storage))
            }
            /// Resets current to head and previous to null; `cursor.init(storage)`.
            pub fn init(&mut self, storage: $borrow) {
                self.raw.init(storage);
            }
            /// Uses the native Initialize spelling; `cursor.initialize(storage)`.
            pub fn initialize(&mut self, storage: $borrow) {
                self.init(storage);
            }
        }
    };
}
initialization!(false, &BaseListStorage);
initialization!(true, &mut BaseListStorage);
impl<N: BasicListNode> BaseListBasicIterator<N, false> {
    /// Copies writable state into a native const iterator; `Self::from_mutable(&cursor)`.
    pub fn from_mutable(other: &BaseListBasicIterator<N, true>) -> Self {
        Self::from_raw(other.raw)
    }
    /// Assigns writable state to a const iterator; `cursor.assign_mutable(&other)`.
    pub fn assign_mutable(&mut self, other: &BaseListBasicIterator<N, true>) -> &mut Self {
        self.raw = other.raw;
        self
    }
}
impl<N: BasicListNode, const M: bool, const O: bool> PartialEq<BaseListBasicIterator<N, O>>
    for BaseListBasicIterator<N, M>
{
    fn eq(&self, other: &BaseListBasicIterator<N, O>) -> bool {
        self.is_equal(other)
    }
}
impl<N: BasicListNode, const M: bool> Eq for BaseListBasicIterator<N, M> {}
