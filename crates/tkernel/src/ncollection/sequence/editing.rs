//! Typed edits delegate topology and cache transitions to the canonical native base.
use super::{Sequence, SequenceCursor, SequenceNodeOwner, no_such_node, out_of_range, owner};
use crate::standard::Failure;
mod removal;
mod transfer;

impl<T> Sequence<T> {
    /// Appends an independently allocated item; `*sequence.append(10) == 10`.
    pub fn append(&mut self, value: T) -> &mut T {
        self.insert_node(self.len(), value)
    }
    /// Prepends an independently allocated item; `*sequence.prepend(10) == 10`.
    pub fn prepend(&mut self, value: T) -> &mut T {
        self.insert_node(0, value)
    }
    /// Inserts before a one-based position; `sequence.insert_before(1, 10)?`.
    pub fn insert_before(&mut self, index: usize, value: T) -> Result<&mut T, Failure> {
        if index == 0 {
            return Err(out_of_range("InsertBefore", index, "an index >= 1"));
        }
        self.insert_after(index - 1, value)
    }
    /// Inserts after an index, with zero meaning prepend; `sequence.insert_after(0, 10)?`.
    pub fn insert_after(&mut self, index: usize, value: T) -> Result<&mut T, Failure> {
        if index > self.len() {
            return Err(out_of_range(
                "InsertAfter",
                index,
                &format!("0..={}", self.len()),
            ));
        }
        Ok(self.insert_node(index, value))
    }
    /// Inserts after the cursor's node; an end cursor prepends; `sequence.insert_after_cursor(&cursor, 10)?`.
    pub fn insert_after_cursor(
        &mut self,
        cursor: &SequenceCursor,
        value: T,
    ) -> Result<&mut T, Failure> {
        self.insert_cursor_with(cursor, || value)
    }
    /// Swaps two nodes using native cache transitions; `sequence.exchange(2, 4)?`.
    pub fn exchange(&mut self, first: usize, second: usize) -> Result<(), Failure> {
        self.offset("Exchange", first)?;
        self.offset("Exchange", second)?;
        unsafe { self.topology.exclusive().exchange(first, second) }
    }
    /// Reverses links and adjusts the native cached index; `sequence.reverse()`.
    pub fn reverse(&mut self) {
        unsafe { self.topology.exclusive().reverse() };
    }
    pub(super) fn cursor_insertion(
        &self,
        member: &str,
        cursor: &SequenceCursor,
    ) -> Result<usize, Failure> {
        if !cursor.more() {
            return Ok(0);
        }
        cursor
            .position(self)
            .map(|index| index + 1)
            .ok_or_else(|| no_such_node(member, *cursor))
    }
    pub(super) fn insert_node(&mut self, offset: usize, value: T) -> &mut T {
        self.insert_node_with(offset, || value)
    }
    pub(super) fn insert_node_with(
        &mut self,
        offset: usize,
        construct: impl FnOnce() -> T,
    ) -> &mut T {
        let node = SequenceNodeOwner::construct(self.allocator(), construct);
        let link = node.link;
        self.owners.push(node);
        // SAFETY: validation precedes construction; owners retain the live inserted allocation.
        unsafe { self.topology.exclusive().insert_after(offset, link) }
            .expect("checked insertion index");
        unsafe { owner::item_mut(link) }
    }
    pub(super) fn insert_cursor_with(
        &mut self,
        cursor: &SequenceCursor,
        construct: impl FnOnce() -> T,
    ) -> Result<&mut T, Failure> {
        self.cursor_insertion("InsertAfter", cursor)?;
        let current = self
            .cursor_node(cursor.address)
            .map_or(std::ptr::null_mut(), |(_, node)| node);
        let raw = super::BaseSequenceRawIterator {
            current,
            previous: std::ptr::null_mut(),
        };
        let node = SequenceNodeOwner::construct(self.allocator(), construct);
        let link = node.link;
        self.owners.push(node);
        unsafe { self.topology.exclusive().insert_after_cursor(&raw, link) };
        Ok(unsafe { owner::item_mut(link) })
    }
}
impl<T: Clone> Sequence<T> {
    /// Allocates before copying the borrowed item; `sequence.append_ref(&10)`.
    pub fn append_ref(&mut self, value: &T) -> &mut T {
        self.insert_node_with(self.len(), || value.clone())
    }
}
