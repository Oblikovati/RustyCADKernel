//! BaseSequence's typed iterator shares the existing detached node state.
use super::{Sequence, SequenceCursor};
use std::marker::PhantomData;

/// Native bidirectional iterator with explicit collection borrows for safe value access.
/// Node accessors return opaque identity tokens; `sequence.base_iterator::<false>(true)`.
/// `MUTABLE=true` enables mutation only when the caller supplies an exclusive sequence borrow.
pub struct BaseSequenceIterator<T, const MUTABLE: bool = false> {
    cursor: SequenceCursor,
    item_type: PhantomData<fn() -> T>,
}

impl<T, const M: bool> Copy for BaseSequenceIterator<T, M> {}
impl<T, const M: bool> Clone for BaseSequenceIterator<T, M> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, const M: bool> Default for BaseSequenceIterator<T, M> {
    fn default() -> Self {
        Self::from_cursor(SequenceCursor::default())
    }
}
impl<T, const M: bool> BaseSequenceIterator<T, M> {
    /// Initializes the first or past-last position; `Self::new(&sequence, false)`.
    pub fn new(sequence: &Sequence<T>, is_start: bool) -> Self {
        Self::from_cursor(SequenceCursor::new(sequence, is_start))
    }
    /// Copies the native base Iterator links without borrowing items; `Self::from_cursor(cursor)`.
    pub fn from_cursor(cursor: SequenceCursor) -> Self {
        Self {
            cursor,
            item_type: PhantomData,
        }
    }
    /// Copies either const specialization, including OCCT's historical reverse conversion.
    /// Example: `let writable = readonly.convert::<true>()`.
    pub fn convert<const OTHER: bool>(&self) -> BaseSequenceIterator<T, OTHER> {
        BaseSequenceIterator::from_cursor(self.cursor)
    }
    /// Copies both links back to the native base Iterator representation; `iterator.copy_to(&mut cursor)`.
    pub fn copy_to(&self, cursor: &mut SequenceCursor) {
        self.cursor.copy_to(cursor);
    }
    /// Assigns either specialization and returns the destination; `writable.assign(&readonly)`.
    pub fn assign<const OTHER: bool>(
        &mut self,
        source: &BaseSequenceIterator<T, OTHER>,
    ) -> &mut Self {
        self.cursor = source.cursor;
        self
    }
    /// Preserves native self-assignment state; `iterator.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Resets the node links to either endpoint; `iterator.init(&sequence, true)`.
    pub fn init(&mut self, sequence: &Sequence<T>, is_start: bool) {
        self.cursor.init(sequence, is_start);
    }
    /// Resets through the native Initialize spelling; `iterator.initialize(&sequence, false)`.
    pub fn initialize(&mut self, sequence: &Sequence<T>, is_start: bool) {
        self.init(sequence, is_start);
    }
    /// Reports a non-null current link without dereferencing it; `iterator.more()`.
    pub fn more(&self) -> bool {
        self.cursor.more()
    }
    /// Reads the current live item, rejecting stale or unrelated node tokens; `iterator.value(&sequence)`.
    pub fn value<'a>(&self, sequence: &'a Sequence<T>) -> Option<&'a T> {
        self.cursor.value(sequence)
    }
    /// Returns native arrow identity after checking membership; `iterator.value_ptr(&sequence)`.
    pub fn value_ptr(&self, sequence: &Sequence<T>) -> Option<*const T> {
        self.value(sequence).map(std::ptr::from_ref)
    }
    /// Advances through the current node order; `iterator.next(&sequence)`.
    pub fn next(&mut self, sequence: &Sequence<T>) {
        self.cursor.next(sequence);
    }
    /// Retreats through the remembered previous link; `iterator.previous(&sequence)`.
    pub fn previous(&mut self, sequence: &Sequence<T>) {
        self.cursor.previous(sequence);
    }
    /// Implements prefix increment and returns this iterator; `iterator.advance(&sequence)`.
    pub fn advance(&mut self, sequence: &Sequence<T>) -> &mut Self {
        self.next(sequence);
        self
    }
    /// Implements postfix increment; `let previous = iterator.post_advance(&sequence)`.
    pub fn post_advance(&mut self, sequence: &Sequence<T>) -> Self {
        let previous = *self;
        self.next(sequence);
        previous
    }
    /// Implements prefix decrement and returns this iterator; `iterator.retreat(&sequence)`.
    pub fn retreat(&mut self, sequence: &Sequence<T>) -> &mut Self {
        self.previous(sequence);
        self
    }
    /// Implements postfix decrement; `let previous = iterator.post_retreat(&sequence)`.
    pub fn post_retreat(&mut self, sequence: &Sequence<T>) -> Self {
        let previous = *self;
        self.previous(sequence);
        previous
    }
    /// Exposes current node identity without exposing link mutation; `iterator.node()`.
    pub fn node(&self) -> Option<usize> {
        self.cursor.current_node()
    }
    /// Exposes the native CurrentNode identity; `iterator.current_node()`.
    pub fn current_node(&self) -> Option<usize> {
        self.node()
    }
    /// Exposes the remembered PreviousNode identity; `iterator.previous_node()`.
    pub fn previous_node(&self) -> Option<usize> {
        self.cursor.previous_node()
    }
    /// Compares current links across const specializations; `writable.is_equal(&readonly)`.
    pub fn is_equal<const OTHER: bool>(&self, other: &BaseSequenceIterator<T, OTHER>) -> bool {
        self.cursor == other.cursor
    }
}
impl<T> BaseSequenceIterator<T, true> {
    /// Mutates a live item under an exclusive sequence borrow; `*iterator.change_value(&mut sequence).unwrap() = item`.
    pub fn change_value<'a>(&self, sequence: &'a mut Sequence<T>) -> Option<&'a mut T> {
        self.cursor.change_value(sequence)
    }
    /// Returns mutable arrow identity after checking membership; `iterator.change_value_ptr(&mut sequence)`.
    pub fn change_value_ptr(&self, sequence: &mut Sequence<T>) -> Option<*mut T> {
        self.change_value(sequence).map(std::ptr::from_mut)
    }
    /// Returns the mutable native node's opaque identity; `iterator.mutable_node()`.
    pub fn mutable_node(&self) -> Option<usize> {
        self.node()
    }
    /// Returns the mutable previous node's opaque identity; `iterator.mutable_previous_node()`.
    pub fn mutable_previous_node(&self) -> Option<usize> {
        self.previous_node()
    }
}
impl<T, const M: bool, const OTHER: bool> PartialEq<BaseSequenceIterator<T, OTHER>>
    for BaseSequenceIterator<T, M>
{
    fn eq(&self, other: &BaseSequenceIterator<T, OTHER>) -> bool {
        self.is_equal(other)
    }
}
impl<T, const M: bool> Eq for BaseSequenceIterator<T, M> {}
