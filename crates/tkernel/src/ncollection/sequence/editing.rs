// Created on: 2002-03-28
// Created by: Alexander GRIGORIEV
// Copyright (c) 2002-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Index- and cursor-positioned sequence edits with OCCT's range errors.
use super::{Sequence, SequenceCursor, SequenceNode, no_such_node, out_of_range};
use crate::standard::Failure;
impl<T> Sequence<T> {
    /// Appends an independently allocated item; `*sequence.append(10) == 10`.
    pub fn append(&mut self, value: T) -> &mut T {
        self.insert_node(self.len(), value)
    }
    /// Prepends an independently allocated item; `*sequence.prepend(10) == 10`.
    pub fn prepend(&mut self, value: T) -> &mut T {
        self.insert_node(0, value)
    }
    /// Inserts before a one-based position; index 0 is OutOfRange; `sequence.insert_before(1, 10)?`.
    pub fn insert_before(&mut self, index: usize, value: T) -> Result<&mut T, Failure> {
        if index == 0 {
            return Err(out_of_range("InsertBefore", index, "an index >= 1"));
        }
        self.insert_after(index - 1, value)
    }
    /// Inserts after an index, with zero meaning prepend; `sequence.insert_after(0, 10)?`.
    pub fn insert_after(&mut self, index: usize, value: T) -> Result<&mut T, Failure> {
        if index > self.len() {
            let expected = format!("0..={}", self.len());
            return Err(out_of_range("InsertAfter", index, &expected));
        }
        Ok(self.insert_node(index, value))
    }
    /// Inserts after the cursor's node; an end cursor prepends, as OCCT's
    /// `InsertAfter(Iterator&, item)` does; `sequence.insert_after_cursor(&cursor, 10)?`.
    pub fn insert_after_cursor(
        &mut self,
        cursor: &SequenceCursor,
        value: T,
    ) -> Result<&mut T, Failure> {
        let offset = self.cursor_insertion("InsertAfter", cursor)?;
        Ok(self.insert_node(offset, value))
    }
    /// Removes an existing item; `sequence.remove(1)?`.
    pub fn remove(&mut self, index: usize) -> Result<(), Failure> {
        let offset = self.offset("Remove", index)?;
        self.nodes.remove(offset);
        Ok(())
    }
    /// Removes an inclusive one-based range, OutOfRange unless `1 <= first <= last <= len`;
    /// `sequence.remove_range(2, 4)?`.
    pub fn remove_range(&mut self, first: usize, last: usize) -> Result<(), Failure> {
        if first == 0 || last > self.len() || first > last {
            let expected = format!("1 <= first <= last <= {}", self.len());
            return Err(out_of_range(
                "Remove",
                first,
                &format!("{expected} (last {last})"),
            ));
        }
        self.nodes.drain(first - 1..last);
        Ok(())
    }
    /// Removes the cursor's node and advances the cursor to its successor; an end cursor is
    /// a no-op (OCCT `Remove(Iterator&)`); `sequence.remove_cursor(&mut cursor)?`.
    pub fn remove_cursor(&mut self, cursor: &mut SequenceCursor) -> Result<(), Failure> {
        if !cursor.more() {
            return Ok(());
        }
        let offset = cursor
            .position(self)
            .ok_or_else(|| no_such_node("Remove", *cursor))?;
        self.nodes.remove(offset);
        cursor.address = self.nodes.get(offset).map(|node| super::address(&**node));
        Ok(())
    }
    /// Frees elements and optionally selects an allocator; `sequence.clear(Some(arena))`.
    pub fn clear(&mut self, allocator: Option<super::BaseAllocator>) {
        self.nodes.clear();
        if let Some(resource) = allocator {
            self.resource = resource;
        }
    }
    /// Swaps two node positions without relocating elements; `sequence.exchange(2, 4)?`.
    pub fn exchange(&mut self, first: usize, second: usize) -> Result<(), Failure> {
        let a = self.offset("Exchange", first)?;
        let b = self.offset("Exchange", second)?;
        self.nodes.swap(a, b);
        Ok(())
    }
    /// Reverses node order without relocating elements; `sequence.reverse()`.
    pub fn reverse(&mut self) {
        self.nodes.reverse();
    }
    /// Moves items `index..` into `destination`, which OCCT clears and rebinds to this
    /// allocator before it checks `index`; `sequence.split(2, &mut tail)?`.
    pub fn split(&mut self, index: usize, destination: &mut Self) -> Result<(), Failure> {
        destination.clear(Some(self.resource.clone()));
        let offset = self.offset("Split", index)?;
        destination.nodes = self.nodes.split_off(offset);
        Ok(())
    }
    /// Moves content while leaving the source empty with its allocator; `Sequence::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        Self {
            resource: source.resource.clone(),
            nodes: std::mem::take(&mut source.nodes),
        }
    }
    /// Replaces ownership from another sequence; `sequence.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = Self::move_from(source);
    }
    /// Vector offset after which a cursor inserts: 0 for an end cursor (prepend).
    pub(super) fn cursor_insertion(
        &self,
        member: &str,
        cursor: &SequenceCursor,
    ) -> Result<usize, Failure> {
        if !cursor.more() {
            return Ok(0);
        }
        let offset = cursor
            .position(self)
            .ok_or_else(|| no_such_node(member, *cursor))?;
        Ok(offset + 1)
    }
    pub(super) fn insert_node(&mut self, offset: usize, value: T) -> &mut T {
        let node = self.resource.allocate_value(SequenceNode {
            value,
            _allocation_identity: 0,
        });
        self.nodes.insert(offset, node);
        &mut self.nodes[offset]
    }
}
