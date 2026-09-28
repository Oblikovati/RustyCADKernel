// Copyright (c) 2025 OPEN CASCADE SAS
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

//! The native `NCollection_PackedMap::Iterator` state, read against the live map.
//!
//! OCCT code such as `Poly_MakeLoops::HeapOfInteger` keeps one iterator while it removes
//! keys from the map. The native iterator stores the current node, bucket and a mask of
//! the bits above the current key, and it reads the node bits again on every step, so a
//! removed key is skipped and the chain is followed through its live links. This cursor
//! stores the same state by value and is advanced against whatever the map holds now.
use super::{PackedInteger, PackedMap, PackedWord};
/// Bucket, node (by block index) and remaining-bit mask of a native iterator; `map.cursor_first()`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PackedCursor {
    bucket: usize,
    block: Option<u64>,
    mask: u64,
}
impl<T: PackedInteger> PackedMap<T> {
    const FULL_MASK: u64 = if T::IS_64_BIT {
        u64::MAX
    } else {
        u32::MAX as u64
    };
    /// Positions a cursor like the native iterator constructor; `let (cursor, key) = map.cursor_first();`.
    pub fn cursor_first(&self) -> (PackedCursor, Option<T>) {
        let mut cursor = PackedCursor {
            bucket: 0,
            block: None,
            mask: Self::FULL_MASK,
        };
        self.cursor_find_first(&mut cursor);
        let key = self.cursor_next(&mut cursor);
        (cursor, key)
    }
    /// Advances like the native `operator++`, skipping keys removed meanwhile; `map.cursor_next(&mut cursor)`.
    pub fn cursor_next(&self, cursor: &mut PackedCursor) -> Option<T> {
        while let Some(block) = cursor.block {
            if let Some(key) = self.cursor_take_bit(cursor, block) {
                return Some(key);
            }
            self.cursor_next_node(cursor, block);
        }
        None
    }
    // findNext: the lowest live bit under the mask, then a mask of the bits above it.
    fn cursor_take_bit(&self, cursor: &mut PackedCursor, block: u64) -> Option<T> {
        let bits = self.block_bits(cursor.bucket, block).unwrap_or(0) & cursor.mask;
        if bits == 0 {
            cursor.mask = Self::FULL_MASK;
            return None;
        }
        let zeros = bits.trailing_zeros();
        cursor.mask = Self::FULL_MASK.checked_shl(zeros + 1).unwrap_or(0) & Self::FULL_MASK;
        Some(T::from_bits((block << Self::SHIFT) | u64::from(zeros)))
    }
    fn block_bits(&self, bucket: usize, block: u64) -> Option<u64> {
        self.buckets
            .get(bucket)?
            .iter()
            .find(|entry| entry.index.bits() == block)
            .map(|entry| entry.bits.bits())
    }
    // nextNode: follow the live chain link, then scan the following buckets.
    fn cursor_next_node(&self, cursor: &mut PackedCursor, block: u64) {
        let chain = self
            .buckets
            .get(cursor.bucket)
            .map_or(&[][..], Vec::as_slice);
        let position = chain.iter().position(|entry| entry.index.bits() == block);
        // A node is only destroyed after the iterator left it, as in every native caller;
        // an unknown node ends its chain instead of reading freed memory.
        if let Some(next) = position.and_then(|position| chain.get(position + 1)) {
            cursor.block = Some(next.index.bits());
            return;
        }
        cursor.bucket += 1;
        self.cursor_find_first(cursor);
    }
    // findFirst: the head of the first non-empty bucket at or after the cursor's bucket.
    fn cursor_find_first(&self, cursor: &mut PackedCursor) {
        cursor.block = None;
        while let Some(chain) = self.buckets.get(cursor.bucket) {
            if let Some(head) = chain.first() {
                cursor.block = Some(head.index.bits());
                return;
            }
            cursor.bucket += 1;
        }
    }
}
