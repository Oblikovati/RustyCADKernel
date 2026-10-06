use super::{Sequence, SequenceCursor, SequenceIter, SequenceIterMut, owner};
use std::marker::PhantomData;

impl SequenceCursor {
    /// Positions at native first or past-last, remembering the final node; `Self::new(&sequence, false)`.
    pub fn new<T>(sequence: &Sequence<T>, is_start: bool) -> Self {
        let (head, tail) = sequence.topology.endpoints();
        let first = pointer_address(head);
        let last = pointer_address(tail);
        if is_start {
            return Self {
                address: first,
                previous: None,
            };
        }
        Self {
            address: None,
            previous: last,
        }
    }
    /// Resets either endpoint; `cursor.init(&sequence, true)`.
    pub fn init<T>(&mut self, sequence: &Sequence<T>, is_start: bool) {
        *self = Self::new(sequence, is_start);
    }
    /// Reports an unexhausted position; `cursor.more()`.
    pub fn more(self) -> bool {
        self.address.is_some()
    }
    /// Borrows a currently owned node's item; `cursor.value(&sequence)`.
    pub fn value<T>(self, sequence: &Sequence<T>) -> Option<&T> {
        let (_, node) = sequence.cursor_node(self.address)?;
        Some(unsafe { owner::item_ref(node) })
    }
    /// Borrows a positioned item exclusively; `*cursor.change_value(&mut sequence)? = 3`.
    pub fn change_value<T>(self, sequence: &mut Sequence<T>) -> Option<&mut T> {
        let (_, node) = sequence.cursor_node(self.address)?;
        Some(unsafe { owner::item_mut(node) })
    }
    /// Follows actual native Next; exhausted cursors remain unchanged; `cursor.next(&sequence)`.
    pub fn next<T>(&mut self, sequence: &Sequence<T>) {
        if self.address.is_none() {
            return;
        }
        let node = sequence.cursor_node(self.address);
        self.previous = self.address;
        self.address = node.and_then(|(_, node)| pointer_address(unsafe { (*node).next() }));
    }
    /// Follows the remembered native previous node; `cursor.previous(&sequence)`.
    pub fn previous<T>(&mut self, sequence: &Sequence<T>) {
        self.address = self.previous;
        if self.address.is_none() {
            return;
        }
        self.previous = sequence
            .cursor_node(self.address)
            .and_then(|(_, node)| pointer_address(unsafe { (*node).previous() }));
    }
    pub(super) fn position<T>(self, sequence: &Sequence<T>) -> Option<usize> {
        sequence.cursor_node(self.address).map(|(index, _)| index)
    }
}
pub(super) fn pointer_address(node: *mut super::SeqNode) -> Option<usize> {
    (!node.is_null()).then_some(node as usize)
}
impl PartialEq for SequenceCursor {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}
impl Eq for SequenceCursor {}
impl<T> Sequence<T> {
    pub(super) fn linked_iter(&self) -> SequenceIter<'_, T> {
        let (front, back) = self.topology.endpoints();
        SequenceIter {
            front,
            back,
            remaining: self.len(),
            item: PhantomData,
        }
    }
    pub(super) fn linked_iter_mut(&mut self) -> SequenceIterMut<'_, T> {
        let base = self.topology.exclusive();
        SequenceIterMut {
            front: base.first_node(),
            back: base.last_node(),
            remaining: base.size(),
            item: PhantomData,
        }
    }
}
impl<'a, T> Iterator for SequenceIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.front;
        self.front = unsafe { (*node).next() };
        self.remaining -= 1;
        Some(unsafe { owner::item_ref(node) })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<T> DoubleEndedIterator for SequenceIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.back;
        self.back = unsafe { (*node).previous() };
        self.remaining -= 1;
        Some(unsafe { owner::item_ref(node) })
    }
}
impl<T> ExactSizeIterator for SequenceIter<'_, T> {}
impl<T> Clone for SequenceIter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            front: self.front,
            back: self.back,
            remaining: self.remaining,
            item: PhantomData,
        }
    }
}
impl<T> PartialEq for SequenceIter<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        let current = (self.remaining > 0).then_some(self.front);
        let other = (other.remaining > 0).then_some(other.front);
        current == other
    }
}
impl<T> Eq for SequenceIter<'_, T> {}
impl<'a, T> Iterator for SequenceIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.front;
        self.front = unsafe { (*node).next() };
        self.remaining -= 1;
        // Only value is borrowed; the link prefix remains available to the iterator.
        Some(unsafe { owner::item_mut(node) })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<T> DoubleEndedIterator for SequenceIterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.back;
        self.back = unsafe { (*node).previous() };
        self.remaining -= 1;
        Some(unsafe { owner::item_mut(node) })
    }
}
impl<T> ExactSizeIterator for SequenceIterMut<'_, T> {}
// SAFETY: lifetimes retain the sequence borrow; links cannot change while iterators exist.
unsafe impl<T: Sync> Send for SequenceIter<'_, T> {}
unsafe impl<T: Sync> Sync for SequenceIter<'_, T> {}
unsafe impl<T: Send> Send for SequenceIterMut<'_, T> {}
unsafe impl<T: Sync> Sync for SequenceIterMut<'_, T> {}
impl<'a, T> IntoIterator for &'a Sequence<T> {
    type Item = &'a T;
    type IntoIter = SequenceIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut Sequence<T> {
    type Item = &'a mut T;
    type IntoIter = SequenceIterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
