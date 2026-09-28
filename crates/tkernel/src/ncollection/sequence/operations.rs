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

use super::{Sequence, SequenceCursor, SequenceNode};
impl<T> Sequence<T> {
    /// Constructs an appended item; `sequence.emplace_append(|| Pair(1,2.0))`.
    pub fn emplace_append(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.append(construct())
    }
    /// Constructs a prepended item; `sequence.emplace_prepend(|| Pair(1,2.0))`.
    pub fn emplace_prepend(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.prepend(construct())
    }
    /// Constructs after a one-based index; `sequence.emplace_after(1, || Pair(1,2.0))`.
    pub fn emplace_after(&mut self, index: usize, construct: impl FnOnce() -> T) -> &mut T {
        self.insert_after(index, construct())
    }
    /// Constructs before a one-based index; `sequence.emplace_before(2, || Pair(1,2.0))`.
    pub fn emplace_before(&mut self, index: usize, construct: impl FnOnce() -> T) -> &mut T {
        self.insert_before(index, construct())
    }
    /// Constructs after a stable node cursor; `sequence.emplace_after_cursor(&cursor, || Pair(1,2.0))`.
    pub fn emplace_after_cursor(
        &mut self,
        cursor: &SequenceCursor,
        construct: impl FnOnce() -> T,
    ) -> &mut T {
        let index = cursor
            .position(self)
            .expect("Sequence cursor missing; expected an existing node")
            + 1;
        self.emplace_after(index, construct)
    }
}
impl<T: Clone> Sequence<T> {
    /// Copies elements into this allocator; `sequence.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        self.clear(None);
        for value in source.iter() {
            self.append(value.clone());
        }
    }
    /// Appends and empties a source sequence; `sequence.append_sequence(&mut source)`.
    pub fn append_sequence(&mut self, source: &mut Self) {
        self.insert_sequence_after(self.len(), source);
    }
    /// Prepends and empties a source, including empty sources (OCC26448); `sequence.prepend_sequence(&mut source)`.
    pub fn prepend_sequence(&mut self, source: &mut Self) {
        self.insert_sequence_after(0, source);
    }
    /// Splices after a one-based index, copying when allocators differ; `sequence.insert_sequence_after(1,&mut source)`.
    pub fn insert_sequence_after(&mut self, index: usize, source: &mut Self) {
        assert!(
            index <= self.len(),
            "Sequence index {index}; expected 0..={}",
            self.len()
        );
        if self.resource == source.resource {
            self.nodes.splice(index..index, source.nodes.drain(..));
            return;
        }
        let copies: Vec<_> = source
            .iter()
            .map(|value| {
                self.resource.allocate_value(SequenceNode {
                    value: value.clone(),
                    _allocation_identity: 0,
                })
            })
            .collect();
        self.nodes.splice(index..index, copies);
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
