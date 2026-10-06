//! The BaseList standard iterator uses the same detached state as its legacy cursor.
use super::{List, ListCursor};
use std::marker::PhantomData;

/// Forward iterator with explicit collection borrows for safe item access.
/// `MUTABLE=true` permits exclusive access; `list.base_iterator::<false>()`.
pub struct BaseListIterator<T, const MUTABLE: bool = false> {
    cursor: ListCursor,
    item_type: PhantomData<fn() -> T>,
}
impl<T, const M: bool> Copy for BaseListIterator<T, M> {}
impl<T, const M: bool> Clone for BaseListIterator<T, M> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, const M: bool> Default for BaseListIterator<T, M> {
    fn default() -> Self {
        Self::from_cursor(ListCursor::default())
    }
}
impl<T, const M: bool> BaseListIterator<T, M> {
    /// Starts at the native first node; `BaseListIterator::<i32>::new(&list)`.
    pub fn new(list: &List<T>) -> Self {
        Self::from_cursor(list.cursor())
    }
    /// Restores both legacy cursor links; `BaseListIterator::<i32>::from_cursor(cursor)`.
    pub fn from_cursor(cursor: ListCursor) -> Self {
        Self {
            cursor,
            item_type: PhantomData,
        }
    }
    /// Exposes a copy for native cursor edit overloads; `let cursor = iterator.cursor()`.
    pub fn cursor(&self) -> ListCursor {
        self.cursor
    }
    /// Exposes both links for edits that update the position; `list.remove(iterator.change_cursor())?`.
    pub fn change_cursor(&mut self) -> &mut ListCursor {
        &mut self.cursor
    }
    /// Converts a mutable specialization into a const iterator; `let readonly = iterator.to_const()`.
    pub fn to_const(&self) -> BaseListIterator<T, false> {
        BaseListIterator::from_cursor(self.cursor)
    }
    /// Copies both native links and returns the destination; `iterator.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        self.cursor = other.cursor;
        self
    }
    /// Preserves self-assignment without borrowing twice; `iterator.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Restores the head and clears the previous link; `iterator.init(&list)`.
    pub fn init(&mut self, list: &List<T>) {
        self.cursor.init(list);
    }
    /// Uses the native Initialize alias; `iterator.initialize(&list)`.
    pub fn initialize(&mut self, list: &List<T>) {
        self.init(list);
    }
    /// Checks the native current link; `iterator.more()`.
    pub fn more(&self) -> bool {
        self.cursor.more()
    }
    /// Borrows the current live value, rejecting stale nodes; `iterator.value(&list)`.
    pub fn value<'a>(&self, list: &'a List<T>) -> Option<&'a T> {
        self.cursor.value(list)
    }
    /// Returns native arrow identity after membership checking; `iterator.value_ptr(&list)`.
    pub fn value_ptr(&self, list: &List<T>) -> Option<*const T> {
        self.cursor.value_ptr(list)
    }
    /// Advances the current/previous links; advancing end is a no-op; `iterator.next(&list)`.
    pub fn next(&mut self, list: &List<T>) {
        self.cursor.next(list);
    }
    /// Returns the destination after prefix increment; `iterator.advance(&list)`.
    pub fn advance(&mut self, list: &List<T>) -> &mut Self {
        self.next(list);
        self
    }
    /// Returns the previous iterator before incrementing; `let old = iterator.post_advance(&list)`.
    pub fn post_advance(&mut self, list: &List<T>) -> Self {
        let old = *self;
        self.next(list);
        old
    }
    /// Implements native Iterator's self-reference; `iterator.iterator()`.
    pub fn iterator(&self) -> &Self {
        self
    }
    /// Implements native ChangeIterator's mutable self-reference; `iterator.change_iterator()`.
    pub fn change_iterator(&mut self) -> &mut Self {
        self
    }
    /// Compares current links across const specializations; `iterator.is_equal(&readonly)`.
    pub fn is_equal<const OTHER: bool>(&self, other: &BaseListIterator<T, OTHER>) -> bool {
        self.cursor == other.cursor
    }
}
impl<T> BaseListIterator<T, false> {
    /// Copies a mutable iterator's state into this const iterator; `readonly.assign_mutable(&writable)`.
    pub fn assign_mutable(&mut self, other: &BaseListIterator<T, true>) -> &mut Self {
        self.cursor = other.cursor;
        self
    }
}
impl<T> BaseListIterator<T, true> {
    /// Borrows a live value exclusively; `*iterator.change_value(&mut list).unwrap() = 7`.
    pub fn change_value<'a>(&self, list: &'a mut List<T>) -> Option<&'a mut T> {
        self.cursor.change_value(list)
    }
    /// Returns native mutable arrow identity after membership checking; `iterator.change_value_ptr(&mut list)`.
    pub fn change_value_ptr(&self, list: &mut List<T>) -> Option<*mut T> {
        self.change_value(list).map(std::ptr::from_mut)
    }
}
impl<T, const M: bool, const OTHER: bool> PartialEq<BaseListIterator<T, OTHER>>
    for BaseListIterator<T, M>
{
    fn eq(&self, other: &BaseListIterator<T, OTHER>) -> bool {
        self.is_equal(other)
    }
}
impl<T, const M: bool> Eq for BaseListIterator<T, M> {}
