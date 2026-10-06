use super::{Sequence, SequenceCursor, out_of_range};
use crate::standard::Failure;
impl<T> Sequence<T> {
    /// Constructs an appended item; `sequence.emplace_append(|| Pair(1,2.0))`.
    pub fn emplace_append(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.insert_node_with(self.len(), construct)
    }
    /// Constructs a prepended item; `sequence.emplace_prepend(|| Pair(1,2.0))`.
    pub fn emplace_prepend(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.insert_node_with(0, construct)
    }
    /// Constructs after a one-based index (0 prepends), checking the index before
    /// constructing as OCCT does; `sequence.emplace_after(1, || Pair(1,2.0))?`.
    pub fn emplace_after(
        &mut self,
        index: usize,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        if index > self.len() {
            let expected = format!("0..={}", self.len());
            return Err(out_of_range("EmplaceAfter", index, &expected));
        }
        Ok(self.insert_node_with(index, construct))
    }
    /// Constructs before a one-based index; `sequence.emplace_before(2, || Pair(1,2.0))?`.
    pub fn emplace_before(
        &mut self,
        index: usize,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        if index == 0 {
            return Err(out_of_range("EmplaceBefore", index, "an index >= 1"));
        }
        self.emplace_after(index - 1, construct)
    }
    /// Constructs after a node cursor; an end cursor prepends (OCCT's
    /// `EmplaceAfter(Iterator&, ...)`); `sequence.emplace_after_cursor(&cursor, || Pair(1,2.0))?`.
    pub fn emplace_after_cursor(
        &mut self,
        cursor: &SequenceCursor,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        let offset = self.cursor_insertion("EmplaceAfter", cursor)?;
        Ok(self.insert_node_with(offset, construct))
    }
}
impl<T: Clone> Sequence<T> {
    /// Copies elements into this allocator; `sequence.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.clear(None);
        for value in source.iter() {
            self.append_ref(value);
        }
        self
    }
    /// Appends and empties a source sequence; `sequence.append_sequence(&mut source)`.
    pub fn append_sequence(&mut self, source: &mut Self) {
        self.splice_sequence(self.len(), source);
    }
    /// Prepends and empties a source, including empty sources (OCC26448); `sequence.prepend_sequence(&mut source)`.
    pub fn prepend_sequence(&mut self, source: &mut Self) {
        self.splice_sequence(0, source);
    }
    /// Splices after a one-based index, copying when allocators differ. As in OCCT, an empty
    /// source returns before the index is checked; `sequence.insert_sequence_after(1,&mut source)?`.
    pub fn insert_sequence_after(
        &mut self,
        index: usize,
        source: &mut Self,
    ) -> Result<(), Failure> {
        if source.is_empty() {
            return Ok(());
        }
        if index > self.len() {
            let expected = format!("0..={}", self.len());
            return Err(out_of_range("InsertAfter", index, &expected));
        }
        self.splice_sequence(index, source);
        Ok(())
    }
    /// Splices before a one-based index; index 0 is OutOfRange even for an empty source;
    /// `sequence.insert_sequence_before(1, &mut source)?`.
    pub fn insert_sequence_before(
        &mut self,
        index: usize,
        source: &mut Self,
    ) -> Result<(), Failure> {
        if index == 0 {
            return Err(out_of_range("InsertBefore", index, "an index >= 1"));
        }
        self.insert_sequence_after(index - 1, source)
    }
    /// Moves (same allocator) or copies and clears (other allocator) `source` after `offset`.
    fn splice_sequence(&mut self, offset: usize, source: &mut Self) {
        if self.resource == source.resource {
            let moved = source.len();
            self.nodes.splice(offset..offset, source.nodes.drain(..));
            self.refresh_links_around(offset);
            self.refresh_links_around(offset + moved);
            return;
        }
        // Native prependSeq/appendSeq link each successful copy before the next copy begins.
        for (index, value) in source.iter().enumerate() {
            self.insert_node_with(offset + index, || value.clone());
        }
        source.clear(None);
    }
}
impl<T: Clone> Clone for Sequence<T> {
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(self.resource.clone());
        copy.assign(self);
        copy
    }
}
