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

use super::{List, ListCursor, find_link};
impl<T> List<T> {
    /// Constructs an appended element from arguments captured in a closure; `list.emplace_append(|| Pair(1,2.0))`.
    pub fn emplace_append(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.append(construct())
    }
    /// Constructs a prepended element; `list.emplace_prepend(|| Pair(1,2.0))`.
    pub fn emplace_prepend(&mut self, construct: impl FnOnce() -> T) -> &mut T {
        self.prepend(construct())
    }
    /// Constructs before a node position; `list.emplace_before(&cursor, || Pair(1,2.0))`.
    pub fn emplace_before(&mut self, cursor: &ListCursor, construct: impl FnOnce() -> T) -> &mut T {
        self.insert_before(construct(), cursor)
    }
    /// Constructs after a node position; `list.emplace_after(&cursor, || Pair(1,2.0))`.
    pub fn emplace_after(&mut self, cursor: &ListCursor, construct: impl FnOnce() -> T) -> &mut T {
        self.insert_after(construct(), cursor)
    }
}
impl<T: Clone> List<T> {
    /// Copies contents while preserving this allocator (OCC25348); `destination.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        self.clear();
        for value in source {
            self.append(value.clone());
        }
    }
    /// Appends and empties another list; `destination.append_list(&mut source)`.
    pub fn append_list(&mut self, source: &mut Self) {
        self.normalize_allocator(source);
        let tail = find_link(&mut self.head, None);
        *tail = source.head.take();
        self.count += std::mem::take(&mut source.count);
    }
    /// Prepends and empties another list; `destination.prepend_list(&mut source)`.
    pub fn prepend_list(&mut self, source: &mut Self) {
        self.normalize_allocator(source);
        let tail = find_link(&mut source.head, None);
        *tail = self.head.take();
        self.head = source.head.take();
        self.count += std::mem::take(&mut source.count);
    }
    /// Splices a list before a positioned node; `destination.insert_list_before(&mut source, &cursor)`.
    pub fn insert_list_before(&mut self, source: &mut Self, cursor: &ListCursor) {
        self.normalize_allocator(source);
        let link = find_link(&mut self.head, cursor.address);
        assert!(
            link.is_some(),
            "List cursor {:?}; expected an existing node",
            cursor
        );
        let tail = find_link(&mut source.head, None);
        *tail = link.take();
        *link = source.head.take();
        self.count += std::mem::take(&mut source.count);
    }
    fn normalize_allocator(&self, source: &mut Self) {
        if self.resource == source.resource {
            return;
        }
        let copy = Self::from_iter_in(source.iter().cloned(), self.resource.clone());
        // OCCT keeps the source's selected allocator when clearing its original nodes.
        let original_resource = source.resource.clone();
        *source = copy;
        source.resource = original_resource;
    }
}
impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        Self::from_iter_in(self.iter().cloned(), self.resource.clone())
    }
}
