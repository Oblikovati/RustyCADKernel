//! Native const-reference overloads must allocate before the user copy constructor.
use super::{List, ListCursor};
use crate::standard::Failure;

impl<T: Clone> List<T> {
    /// Copies into a preallocated tail node; `list.append_ref(&value)`.
    pub fn append_ref(&mut self, value: &T) -> &mut T {
        self.emplace_append(|| value.clone())
    }
    /// Copies into a preallocated head node; `list.prepend_ref(&value)`.
    pub fn prepend_ref(&mut self, value: &T) -> &mut T {
        self.emplace_prepend(|| value.clone())
    }
    /// Copies before validating the native position; `list.insert_before_ref(&value, &cursor)?`.
    pub fn insert_before_ref(&mut self, value: &T, cursor: &ListCursor) -> Result<&mut T, Failure> {
        self.emplace_before(cursor, || value.clone())
    }
    /// Copies before validating the native position; `list.insert_after_ref(&value, &cursor)?`.
    pub fn insert_after_ref(&mut self, value: &T, cursor: &ListCursor) -> Result<&mut T, Failure> {
        self.emplace_after(cursor, || value.clone())
    }
    /// Copies and points the cursor at the appended node; `list.append_at_ref(&value, &mut cursor)`.
    pub fn append_at_ref(&mut self, value: &T, cursor: &mut ListCursor) {
        self.append_constructed_at(cursor, || value.clone());
    }
}
