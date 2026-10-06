use super::{Sequence, SequenceCursor, SequenceIter, SequenceIterMut, address};
impl SequenceCursor {
    /// Positions on the first item, or past the last one with `previous` on the last item
    /// (OCCT's `Iterator(seq, isStart)`); `SequenceCursor::new(&sequence, false)`.
    pub fn new<T>(sequence: &Sequence<T>, is_start: bool) -> Self {
        let first = sequence.nodes.first().map(|node| address(&**node));
        let last = sequence.nodes.last().map(|node| address(&**node));
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
    /// Repositions as a fresh cursor would (`Iterator::Init`); `cursor.init(&sequence, true)`.
    pub fn init<T>(&mut self, sequence: &Sequence<T>, is_start: bool) {
        *self = Self::new(sequence, is_start);
    }
    /// Reports whether a position remains; `cursor.more()`.
    pub fn more(self) -> bool {
        self.address.is_some()
    }
    /// Borrows the positioned element; `cursor.value(&sequence)`.
    pub fn value<T>(self, sequence: &Sequence<T>) -> Option<&T> {
        self.position(sequence)
            .map(|index| &sequence.nodes[index].value)
    }
    /// Borrows the positioned element exclusively; `*cursor.change_value(&mut sequence).unwrap()=42`.
    pub fn change_value<T>(self, sequence: &mut Sequence<T>) -> Option<&mut T> {
        let index = self.position(sequence)?;
        Some(&mut sequence.nodes[index].value)
    }
    /// Advances through the current node order; at the end it stays put, as OCCT's
    /// `operator++` does; `cursor.next(&sequence)`.
    pub fn next<T>(&mut self, sequence: &Sequence<T>) {
        if self.address.is_none() {
            return;
        }
        let index = self.position(sequence);
        self.previous = self.address;
        self.address = index
            .and_then(|index| sequence.nodes.get(index + 1))
            .map(|node| address(&**node));
    }
    /// Steps back to the remembered previous node (OCCT's `Iterator::Previous`), so a cursor
    /// past the end returns to the last item; `cursor.previous(&sequence)`.
    pub fn previous<T>(&mut self, sequence: &Sequence<T>) {
        self.address = self.previous;
        if self.address.is_none() {
            return;
        }
        self.previous = self
            .position(sequence)
            .and_then(|index| index.checked_sub(1))
            .map(|index| address(&*sequence.nodes[index]));
    }
    pub(super) fn position<T>(self, sequence: &Sequence<T>) -> Option<usize> {
        sequence
            .nodes
            .iter()
            .position(|node| Some(address(&**node)) == self.address)
    }
}
impl PartialEq for SequenceCursor {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}
impl Eq for SequenceCursor {}
impl<'a, T> Iterator for SequenceIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.next().map(|node| &node.value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.remaining.size_hint()
    }
}
impl<T> DoubleEndedIterator for SequenceIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.remaining.next_back().map(|node| &node.value)
    }
}
impl<T> ExactSizeIterator for SequenceIter<'_, T> {}
impl<T> Clone for SequenceIter<'_, T> {
    fn clone(&self) -> Self {
        Self {
            remaining: self.remaining.clone(),
        }
    }
}
impl<T> PartialEq for SequenceIter<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        // OCCT compares the current node, including null ends from unrelated sequences.
        let current = self
            .remaining
            .as_slice()
            .first()
            .map(|node| address(&**node));
        let other = other
            .remaining
            .as_slice()
            .first()
            .map(|node| address(&**node));
        current == other
    }
}
impl<T> Eq for SequenceIter<'_, T> {}
impl<'a, T> Iterator for SequenceIterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.next().map(|node| &mut node.value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.remaining.size_hint()
    }
}
impl<T> DoubleEndedIterator for SequenceIterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.remaining.next_back().map(|node| &mut node.value)
    }
}
impl<T> ExactSizeIterator for SequenceIterMut<'_, T> {}
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
