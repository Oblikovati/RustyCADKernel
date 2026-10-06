//! Checked native access and explicit cursor expressions that Rust syntax cannot borrow directly.
use super::{List, ListCursor, ListIter, no_such_object};
use crate::standard::Failure;

impl<T> List<T> {
    /// Returns the native unsigned `Size`; `assert_eq!(list.size(), list.len())`.
    pub fn size(&self) -> usize {
        self.len()
    }
    /// Checks native `First`'s exception domain; `let first = list.first_checked()?`.
    pub fn first_checked(&self) -> Result<&T, Failure> {
        self.first()
            .ok_or_else(|| no_such_object("First", "an empty list"))
    }
    /// Checks mutable native `First`; `*list.first_mut_checked()? = 3`.
    pub fn first_mut_checked(&mut self) -> Result<&mut T, Failure> {
        self.first_mut()
            .ok_or_else(|| no_such_object("First", "an empty list"))
    }
    /// Checks native `Last`'s exception domain; `let last = list.last_checked()?`.
    pub fn last_checked(&self) -> Result<&T, Failure> {
        self.last()
            .ok_or_else(|| no_such_object("Last", "an empty list"))
    }
    /// Checks mutable native `Last`; `*list.last_mut_checked()? = 3`.
    pub fn last_mut_checked(&mut self) -> Result<&mut T, Failure> {
        self.last_mut()
            .ok_or_else(|| no_such_object("Last", "an empty list"))
    }
    /// Preserves native copy/move self-assignment's node identity; `list.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}

impl ListCursor {
    /// Constructs the legacy iterator at the head; `let cursor = ListCursor::new(&list)`.
    pub fn new<T>(list: &List<T>) -> Self {
        list.cursor()
    }
    /// Resets native `Initialize` to the head; `cursor.initialize(&list)`.
    pub fn initialize<T>(&mut self, list: &List<T>) {
        self.init(list);
    }
    /// Copies a const STL iterator's node position; `ListCursor::from_iterator(&list.iter())`.
    pub fn from_iterator<T>(iterator: &ListIter<'_, T>) -> Self {
        Self {
            address: super::address(iterator.next),
            previous: iterator.previous,
        }
    }
    /// Assigns a const STL iterator's position and returns the destination; `cursor.assign_iterator(&list.iter())`.
    pub fn assign_iterator<T>(&mut self, iterator: &ListIter<'_, T>) -> &mut Self {
        *self = Self::from_iterator(iterator);
        self
    }
    /// Copies native cursor state and returns the destination; `cursor.assign(&other)`.
    pub fn assign(&mut self, other: &Self) -> &mut Self {
        *self = *other;
        self
    }
    /// Compares node identities, including universal end cursors; `cursor.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self == other
    }
    /// Implements prefix increment, including the native end no-op; `cursor.advance(&list)`.
    pub fn advance<T>(&mut self, list: &List<T>) -> &mut Self {
        self.next(list);
        self
    }
    /// Returns the old position before advancing; `let previous = cursor.post_advance(&list)`.
    pub fn post_advance<T>(&mut self, list: &List<T>) -> Self {
        let previous = *self;
        self.next(list);
        previous
    }
    /// Exposes native arrow identity while checking membership; `cursor.value_ptr(&list)`.
    pub fn value_ptr<T>(&self, list: &List<T>) -> Option<*const T> {
        self.value(list).map(std::ptr::from_ref)
    }
}

impl<T> Copy for ListIter<'_, T> {}
impl<T> Clone for ListIter<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Default for ListIter<'_, T> {
    fn default() -> Self {
        Self {
            next: std::ptr::null_mut(),
            item: std::marker::PhantomData,
            previous: None,
        }
    }
}
