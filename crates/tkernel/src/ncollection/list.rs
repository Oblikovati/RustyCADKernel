// Created on: 2002-04-17
// Created by: Alexander Kartomin (akm)
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

//! Singly linked OCCT lists with allocator-owned stable nodes.
use super::{AllocatedBox, BaseAllocator};
type Link<T> = Option<AllocatedBox<Node<T>>>;
struct Node<T> {
    value: T,
    next: Link<T>,
}
/// Allocator-owned list; `List::from_iter_in([1,2], BaseAllocator::common())`.
pub struct List<T> {
    resource: BaseAllocator,
    head: Link<T>,
    count: usize,
}
/// Detached node position, usable across insertions; `list.cursor().value(&list)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListCursor {
    address: Option<usize>,
}
/// Forward shared iterator; `list.iter()`.
pub struct ListIter<'a, T> {
    next: Option<&'a Node<T>>,
}
/// Forward exclusive iterator; `list.iter_mut()`.
pub struct ListIterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}
impl<T> List<T> {
    /// Creates an empty list with the chosen resource; `List::<i32>::with_allocator(arena)`.
    pub fn with_allocator(resource: BaseAllocator) -> Self {
        Self {
            resource,
            head: None,
            count: 0,
        }
    }
    /// Constructs items in a selected resource; `List::from_iter_in([1,2], arena)`.
    pub fn from_iter_in(values: impl IntoIterator<Item = T>, resource: BaseAllocator) -> Self {
        let mut list = Self::with_allocator(resource);
        for value in values {
            list.append(value);
        }
        list
    }
    /// Returns allocator identity; `list.allocator() == &arena`.
    pub fn allocator(&self) -> &BaseAllocator {
        &self.resource
    }
    /// Returns the element count; `list.len() == 2`.
    pub fn len(&self) -> usize {
        self.count
    }
    /// Returns OCCT's signed extent; `list.extent() == 2`.
    pub fn extent(&self) -> i32 {
        self.count as i32
    }
    /// Checks whether the chain is empty; `list.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// Borrows the head; `list.first() == Some(&1)`.
    pub fn first(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }
    /// Borrows the head for mutation; `*list.first_mut().unwrap() = 42`.
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.value)
    }
    /// Borrows the final element; `list.last() == Some(&2)`.
    pub fn last(&self) -> Option<&T> {
        self.iter().last()
    }
    /// Borrows all elements in chain order; `list.iter().count()`.
    pub fn iter(&self) -> ListIter<'_, T> {
        ListIter {
            next: self.head.as_deref(),
        }
    }
    /// Borrows exclusive elements in chain order; `list.iter_mut().for_each(|v| *v += 1)`.
    pub fn iter_mut(&mut self) -> ListIterMut<'_, T> {
        ListIterMut {
            next: self.head.as_deref_mut(),
        }
    }
    /// Returns a stable position at the head; `let mut cursor = list.cursor()`.
    pub fn cursor(&self) -> ListCursor {
        ListCursor {
            address: address(&self.head),
        }
    }
    /// Appends a freshly allocated node; `*list.append(42) == 42`.
    pub fn append(&mut self, value: T) -> &mut T {
        let node = self.resource.allocate_value(Node { value, next: None });
        self.count += 1;
        let mut tail = &mut self.head;
        while let Some(node) = tail {
            tail = &mut node.next;
        }
        &mut tail.insert(node).value
    }
    /// Prepends a freshly allocated node; `*list.prepend(42) == 42`.
    pub fn prepend(&mut self, value: T) -> &mut T {
        let node = self.resource.allocate_value(Node {
            value,
            next: self.head.take(),
        });
        self.count += 1;
        &mut self.head.insert(node).value
    }
    /// Keeps selected nodes without reallocating survivors; `list.retain(|value| *value > 0)`.
    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        let mut link = &mut self.head;
        while let Some(node) = link.as_ref() {
            if keep(&node.value) {
                link = &mut link.as_mut().unwrap().next;
                continue;
            }
            let mut removed = link.take().unwrap();
            *link = removed.next.take();
            self.count -= 1;
        }
    }
    /// Removes the head if present; `list.remove_first()`.
    pub fn remove_first(&mut self) {
        if let Some(mut node) = self.head.take() {
            self.head = node.next.take();
            self.count -= 1;
        }
    }
    /// Frees all nodes without recursive destruction; `list.clear()`.
    pub fn clear(&mut self) {
        while self.head.is_some() {
            self.remove_first();
        }
    }
    /// Removes a positioned node and advances the cursor; `list.remove(&mut cursor)`.
    pub fn remove(&mut self, cursor: &mut ListCursor) {
        let link = find_link(&mut self.head, cursor.address);
        let Some(mut node) = link.take() else {
            return;
        };
        *link = node.next.take();
        cursor.address = address(link);
        self.count -= 1;
    }
    /// Inserts immediately before a cursor; `list.insert_before(20, &cursor)`.
    pub fn insert_before(&mut self, value: T, cursor: &ListCursor) -> &mut T {
        assert!(
            cursor.more(),
            "List cursor is at end; expected an existing node"
        );
        let mut node = self.resource.allocate_value(Node { value, next: None });
        let link = find_link(&mut self.head, cursor.address);
        assert!(
            link.is_some(),
            "List cursor {:?}; expected a node in this list",
            cursor
        );
        node.next = link.take();
        self.count += 1;
        &mut link.insert(node).value
    }
    /// Inserts immediately after a cursor; `list.insert_after(20, &cursor)`.
    pub fn insert_after(&mut self, value: T, cursor: &ListCursor) -> &mut T {
        let mut node = self.resource.allocate_value(Node { value, next: None });
        let existing = find_link(&mut self.head, cursor.address)
            .as_mut()
            .expect("List cursor missing; expected an existing node");
        node.next = existing.next.take();
        self.count += 1;
        &mut existing.next.insert(node).value
    }
    /// Reverses links without moving values between nodes; `list.reverse()`.
    pub fn reverse(&mut self) {
        let mut reversed = None;
        while let Some(mut node) = self.head.take() {
            self.head = node.next.take();
            node.next = reversed;
            reversed = Some(node);
        }
        self.head = reversed;
    }
    /// Exchanges nodes and their allocator resources; `first.exchange(&mut second)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// Transfers ownership, leaving an empty source; `List::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// Replaces this list with source ownership; `list.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = Self::move_from(source);
    }
    /// Allocator-container spelling of append; `list.push_back(42)`.
    pub fn push_back(&mut self, value: T) {
        self.append(value);
    }
    /// Allocator-container spelling of first; `list.front() == Some(&42)`.
    pub fn front(&self) -> Option<&T> {
        self.first()
    }
}
impl<T: PartialEq> List<T> {
    /// Removes the first equal value; `list.remove_value(&42)`.
    pub fn remove_value(&mut self, value: &T) -> bool {
        let mut cursor = self.cursor();
        while cursor.more() {
            if cursor.value(self) == Some(value) {
                self.remove(&mut cursor);
                return true;
            }
            cursor.next(self);
        }
        false
    }
}
impl<T> Default for List<T> {
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        self.clear();
    }
}
impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter_in(iter, BaseAllocator::common())
    }
}
fn address<T>(link: &Link<T>) -> Option<usize> {
    link.as_ref().map(|node| &**node as *const Node<T> as usize)
}
fn find_link<T>(mut link: &mut Link<T>, target: Option<usize>) -> &mut Link<T> {
    while link.is_some() && address(link) != target {
        link = &mut link.as_mut().unwrap().next;
    }
    link
}
mod iteration;
mod operations;
