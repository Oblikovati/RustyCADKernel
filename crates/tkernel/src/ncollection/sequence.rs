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

//! One-based sequences retain element allocations while rearranging their order.
use super::{AllocatedBox, BaseAllocator};
// OCCT allocates a node even for empty C++ value types. Keep Rust zero-sized
// values in nonzero storage so detached cursors still identify distinct nodes.
struct SequenceNode<T> {
    value: T,
    _allocation_identity: u8,
}
impl<T> std::ops::Deref for SequenceNode<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T> std::ops::DerefMut for SequenceNode<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
/// Stable allocator-owned elements with one-based access; `Sequence::default().append(42)`.
pub struct Sequence<T> {
    resource: BaseAllocator,
    nodes: Vec<AllocatedBox<SequenceNode<T>>>,
}
/// Shared bidirectional traversal; `sequence.iter().rev()`.
pub struct SequenceIter<'a, T> {
    remaining: std::slice::Iter<'a, AllocatedBox<SequenceNode<T>>>,
}
/// Exclusive bidirectional traversal; `sequence.iter_mut().next_back()`.
pub struct SequenceIterMut<'a, T> {
    remaining: std::slice::IterMut<'a, AllocatedBox<SequenceNode<T>>>,
}
/// Detached stable position; `sequence.cursor().value(&sequence)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequenceCursor {
    address: Option<usize>,
}
impl<T> Sequence<T> {
    /// Creates an empty sequence with a chosen allocator; `Sequence::<i32>::with_allocator(arena)`.
    pub fn with_allocator(resource: BaseAllocator) -> Self {
        Self {
            resource,
            nodes: Vec::new(),
        }
    }
    /// Returns selected allocator identity; `sequence.allocator() == &arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.resource
    }
    /// Returns the element count; `sequence.len() == 3`.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    /// Returns the signed length; `sequence.length() == 3`.
    pub fn length(&self) -> i32 {
        self.nodes.len() as i32
    }
    /// Checks for an empty sequence; `sequence.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    /// Returns the fixed lower bound; `sequence.lower() == 1`.
    pub const fn lower(&self) -> usize {
        1
    }
    /// Returns the upper bound, zero when empty; `sequence.upper() == sequence.len()`.
    pub fn upper(&self) -> usize {
        self.len()
    }
    /// Borrows the first item; `sequence.first() == Some(&10)`.
    pub fn first(&self) -> Option<&T> {
        self.nodes.first().map(|node| &node.value)
    }
    /// Borrows the final item; `sequence.last() == Some(&30)`.
    pub fn last(&self) -> Option<&T> {
        self.nodes.last().map(|node| &node.value)
    }
    /// Borrows an item by one-based index; `sequence.value(1)`.
    pub fn value(&self, index: usize) -> &T {
        &self.nodes[self.offset(index)]
    }
    /// Borrows an item exclusively by one-based index; `*sequence.change_value(1)=10`.
    pub fn change_value(&mut self, index: usize) -> &mut T {
        let offset = self.offset(index);
        &mut self.nodes[offset]
    }
    /// Replaces an existing item; `sequence.set_value(1, 10)`.
    pub fn set_value(&mut self, index: usize, value: T) {
        *self.change_value(index) = value;
    }
    /// Appends an independently allocated item; `sequence.append(10)`.
    pub fn append(&mut self, value: T) -> &mut T {
        self.insert_after(self.len(), value)
    }
    /// Prepends an independently allocated item; `sequence.prepend(10)`.
    pub fn prepend(&mut self, value: T) -> &mut T {
        self.insert_after(0, value)
    }
    /// Inserts before a one-based position; `sequence.insert_before(1, 10)`.
    pub fn insert_before(&mut self, index: usize, value: T) -> &mut T {
        assert!(
            index > 0,
            "Sequence index {index}; expected 1..={}",
            self.len() + 1
        );
        self.insert_after(index - 1, value)
    }
    /// Inserts after an index, with zero meaning prepend; `sequence.insert_after(0, 10)`.
    pub fn insert_after(&mut self, index: usize, value: T) -> &mut T {
        assert!(
            index <= self.len(),
            "Sequence index {index}; expected 0..={}",
            self.len()
        );
        self.nodes.insert(
            index,
            self.resource.allocate_value(SequenceNode {
                value,
                _allocation_identity: 0,
            }),
        );
        &mut self.nodes[index]
    }
    /// Removes an existing item; `sequence.remove(1)`.
    pub fn remove(&mut self, index: usize) {
        let offset = self.offset(index);
        self.nodes.remove(offset);
    }
    /// Removes an inclusive one-based range; `sequence.remove_range(2, 4)`.
    pub fn remove_range(&mut self, first: usize, last: usize) {
        let start = self.offset(first);
        let end = self.offset(last);
        assert!(
            start <= end,
            "Sequence range {first}..={last}; expected ascending bounds"
        );
        self.nodes.drain(start..=end);
    }
    /// Frees elements and optionally selects an allocator; `sequence.clear(Some(arena))`.
    pub fn clear(&mut self, allocator: Option<BaseAllocator>) {
        self.nodes.clear();
        if let Some(resource) = allocator {
            self.resource = resource;
        }
    }
    /// Swaps two node positions without relocating elements; `sequence.exchange(2, 4)`.
    pub fn exchange(&mut self, first: usize, second: usize) {
        let a = self.offset(first);
        let b = self.offset(second);
        self.nodes.swap(a, b);
    }
    /// Reverses node order without relocating elements; `sequence.reverse()`.
    pub fn reverse(&mut self) {
        self.nodes.reverse();
    }
    /// Transfers the tail into a cleared destination; `sequence.split(2, &mut tail)`.
    pub fn split(&mut self, index: usize, destination: &mut Self) {
        let offset = self.offset(index);
        destination.clear(Some(self.resource.clone()));
        destination.nodes = self.nodes.split_off(offset);
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
    /// Borrows elements in order; `sequence.iter().sum::<i32>()`.
    pub fn iter(&self) -> SequenceIter<'_, T> {
        SequenceIter {
            remaining: self.nodes.iter(),
        }
    }
    /// Borrows exclusive elements in order; `sequence.iter_mut().for_each(|v| *v+=1)`.
    pub fn iter_mut(&mut self) -> SequenceIterMut<'_, T> {
        SequenceIterMut {
            remaining: self.nodes.iter_mut(),
        }
    }
    /// Captures a position at the first element; `let cursor=sequence.cursor()`.
    pub fn cursor(&self) -> SequenceCursor {
        SequenceCursor {
            address: self.nodes.first().map(|node| address(&**node)),
        }
    }
    fn offset(&self, index: usize) -> usize {
        assert!(
            index >= 1 && index <= self.len(),
            "Sequence index {index}; expected 1..={}",
            self.len()
        );
        index - 1
    }
}
impl<T> Default for Sequence<T> {
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}
impl<T> FromIterator<T> for Sequence<T> {
    fn from_iter<I: IntoIterator<Item = T>>(items: I) -> Self {
        let mut sequence = Self::default();
        for item in items {
            sequence.append(item);
        }
        sequence
    }
}
impl<T> std::ops::Index<usize> for Sequence<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.value(index)
    }
}
impl<T> std::ops::IndexMut<usize> for Sequence<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.change_value(index)
    }
}
fn address<T>(value: &T) -> usize {
    value as *const T as usize
}
mod iteration;
mod operations;
