// Copyright (c) 2009-2014 OPEN CASCADE SAS
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
// Rust translation of Poly_MakeLoops::HeapOfInteger in FoundationClasses/TKMath/Poly/Poly_MakeLoops.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Poly_MakeLoops::HeapOfInteger`: a packed set whose "top" is the key under a cursor kept
//! across removals, so the loop search visits start links in the native map order.
use rustycad_tkernel::{
    ncollection::{PackedCursor, PackedMap},
    standard::Failure,
};
/// Integer set with a persistent iteration cursor; `HeapOfInteger::new(4000)`.
pub struct HeapOfInteger {
    map: PackedMap<i32>,
    cursor: Option<(PackedCursor, Option<i32>)>,
}
impl Default for HeapOfInteger {
    fn default() -> Self {
        Self::new(1)
    }
}
impl HeapOfInteger {
    /// An empty set with a bucket-count hint; `HeapOfInteger::new(4000)`.
    pub fn new(buckets: i32) -> Self {
        Self {
            map: PackedMap::new(usize::try_from(buckets).unwrap_or(0)),
            cursor: None,
        }
    }
    /// Removes every key and forgets the cursor; `heap.clear()`.
    pub fn clear(&mut self) {
        self.map.clear();
        self.cursor = None;
    }
    /// Adds a key; the cursor restarts at the next query; `heap.add(5)`.
    pub fn add(&mut self, value: i32) {
        self.map.add(value);
        self.cursor = None;
    }
    /// The key under the cursor, or `Standard_NoSuchObject` when none is left; `heap.top()?`.
    pub fn top(&mut self) -> Result<i32, Failure> {
        self.current().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "NCollection_PackedMap::Iterator::Key: exhausted iterator; expected a remaining key",
                "",
            )
        })
    }
    /// Number of keys; `heap.extent()`.
    pub fn extent(&self) -> i32 {
        self.map.extent()
    }
    /// Membership test; `heap.contains(5)`.
    pub fn contains(&self, value: i32) -> bool {
        self.map.contains(value)
    }
    /// Removes a key, first stepping the cursor past it when it is the current one;
    /// `heap.remove(5)`.
    pub fn remove(&mut self, value: i32) {
        if let Some((cursor, key)) = &mut self.cursor
            && *key == Some(value)
        {
            *key = self.map.cursor_next(cursor);
        }
        self.map.remove(value);
    }
    /// True when the cursor has no key left; `heap.is_empty()`.
    pub fn is_empty(&mut self) -> bool {
        self.current().is_none()
    }
    fn current(&mut self) -> Option<i32> {
        let map = &self.map;
        self.cursor.get_or_insert_with(|| map.cursor_first()).1
    }
}
