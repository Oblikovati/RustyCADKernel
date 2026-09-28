// Copyright (c) 2002-2025 OPEN CASCADE SAS
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

//! `NCollection_DoubleMap::Iterator`: the More/Next cursor over first-key bucket order.
use super::super::base_map::exhausted_iterator;
use super::{Debug, DoubleMap, DoubleMapItems, HashPolicy, Pair};
use crate::standard::Failure;

/// A read-only OCCT `DoubleMap::Iterator`; `let mut it = map.cursor();`.
pub struct DoubleMapIterator<'a, A, B> {
    pairs: DoubleMapItems<'a, A, B>,
}

impl<A: Debug, B: Debug, H1: HashPolicy<A>, H2: HashPolicy<B>> DoubleMap<A, B, H1, H2> {
    /// `Iterator(const NCollection_DoubleMap&)`: a cursor on the first pair in first-key bucket order.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DoubleMap;
    /// let mut map = DoubleMap::<i32, i32>::default();
    /// map.bind(1, 5).unwrap();
    /// let it = map.cursor();
    /// assert_eq!((it.key1().unwrap(), it.key2().unwrap()), (&1, &5));
    /// ```
    pub fn cursor(&self) -> DoubleMapIterator<'_, A, B> {
        DoubleMapIterator {
            pairs: self.items(),
        }
    }
}

impl<'a, A, B> DoubleMapIterator<'a, A, B> {
    /// `More`: true while the cursor is on a pair.
    pub fn more(&self) -> bool {
        self.pairs.current().is_some()
    }
    /// `Next`: moves to the following pair.
    pub fn next(&mut self) {
        self.pairs.next();
    }
    /// `Key1`: the current first key, or Standard_NoSuchObject past the end.
    pub fn key1(&self) -> Result<&'a A, Failure> {
        Ok(&self.pair("NCollection_DoubleMap::Iterator::Key1")?.first)
    }
    /// `Key2`: the current second key, or Standard_NoSuchObject past the end.
    pub fn key2(&self) -> Result<&'a B, Failure> {
        Ok(&self.pair("NCollection_DoubleMap::Iterator::Key2")?.second)
    }
    /// `Value`: the current second key (the map's value type), or Standard_NoSuchObject.
    pub fn value(&self) -> Result<&'a B, Failure> {
        Ok(&self.pair("NCollection_DoubleMap::Iterator::Value")?.second)
    }
    /// `Initialize(const NCollection_DoubleMap&)`: restarts on another map.
    pub fn initialize<H1: HashPolicy<A>, H2: HashPolicy<B>>(
        &mut self,
        map: &'a DoubleMap<A, B, H1, H2>,
    ) where
        A: Debug,
        B: Debug,
    {
        *self = map.cursor();
    }
    /// `Reset`: restarts on the map the cursor was created for.
    pub fn reset(&mut self) {
        self.pairs.bucket = 0;
        self.pairs.index = 0;
        self.pairs.seek();
    }
    /// `IsEqual`: both cursors are on the same pair, or both are past the end.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.pairs == other.pairs
    }
    fn pair(&self, method: &str) -> Result<&'a Pair<A, B>, Failure> {
        let slot = self
            .pairs
            .buckets
            .get(self.pairs.bucket)
            .and_then(|bucket| bucket.get(self.pairs.index));
        let pair = slot.and_then(|&slot| self.pairs.slots[slot].as_ref());
        pair.map(|node| &**node)
            .ok_or_else(|| exhausted_iterator(method))
    }
}

/// `Iterator()`: a cursor attached to no map, whose `more()` is false.
impl<A, B> Default for DoubleMapIterator<'_, A, B> {
    fn default() -> Self {
        Self {
            pairs: DoubleMapItems {
                slots: &[],
                buckets: &[],
                bucket: 0,
                index: 0,
            },
        }
    }
}
