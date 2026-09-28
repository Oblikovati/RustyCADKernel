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

use super::{Sequence, SequenceCursor, SequenceIter, SequenceIterMut, address};
impl SequenceCursor {
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
    /// Advances through the current node order; `cursor.next(&sequence)`.
    pub fn next<T>(&mut self, sequence: &Sequence<T>) {
        self.address = self
            .position(sequence)
            .and_then(|index| sequence.nodes.get(index + 1))
            .map(|node| address(&**node));
    }
    pub(super) fn position<T>(self, sequence: &Sequence<T>) -> Option<usize> {
        sequence
            .nodes
            .iter()
            .position(|node| Some(address(&**node)) == self.address)
    }
}
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
impl<T> PartialEq for SequenceIter<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.remaining.as_slice().as_ptr() == other.remaining.as_slice().as_ptr()
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
