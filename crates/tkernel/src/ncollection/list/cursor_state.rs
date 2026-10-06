//! Native BaseList current/previous state without dereferencing detached addresses.
use super::{BaseListIterator, List, ListCursor, ListIterMut, Node};
use crate::standard::Failure;

impl PartialEq for ListCursor {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}
impl Eq for ListCursor {}
impl ListCursor {
    /// Returns the opaque native current-node identity; `cursor.current_node()`.
    pub fn current_node(&self) -> Option<usize> {
        self.address
    }
    /// Returns the last remembered predecessor, including at end; `cursor.previous_node()`.
    pub fn previous_node(&self) -> Option<usize> {
        self.previous
    }
    /// Copies a mutable standard iterator's two links; `ListCursor::from_mut_iterator(&iterator)`.
    pub fn from_mut_iterator<T>(iterator: &ListIterMut<'_, T>) -> Self {
        Self {
            address: iterator
                .next
                .as_deref()
                .map(|node| node as *const Node<T> as usize),
            previous: iterator.previous,
        }
    }
}
impl<T> List<T> {
    /// Constructs a detached native standard iterator; `list.base_iterator::<false>()`.
    pub fn base_iterator<const MUTABLE: bool>(&self) -> BaseListIterator<T, MUTABLE> {
        BaseListIterator::new(self)
    }
    /// Inserts an owned item and updates the native previous link; `list.insert_before_at(7, &mut cursor)?`.
    pub fn insert_before_at(
        &mut self,
        value: T,
        cursor: &mut ListCursor,
    ) -> Result<&mut T, Failure> {
        self.emplace_before_at(cursor, || value)
    }
}
impl<T: Clone> List<T> {
    /// Copies after allocation and updates the native previous link; `list.insert_before_at_ref(&7, &mut cursor)?`.
    pub fn insert_before_at_ref(
        &mut self,
        value: &T,
        cursor: &mut ListCursor,
    ) -> Result<&mut T, Failure> {
        self.emplace_before_at(cursor, || value.clone())
    }
}
