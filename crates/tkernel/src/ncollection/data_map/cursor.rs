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

//! `NCollection_DataMap::Iterator`: the More/Next/Value cursor over bucket order.
use super::super::base_map::exhausted_iterator;
use super::{Bucket, DataMap, DataMapItems, Debug, Entry, HashPolicy};
use crate::standard::Failure;

/// The first occupied `(bucket, index)` at or after the given one, or the end position.
pub(super) fn settle<K, V>(
    buckets: &[Bucket<K, V>],
    mut bucket: usize,
    mut index: usize,
) -> (usize, usize) {
    while bucket < buckets.len() && index >= buckets[bucket].len() {
        bucket += 1;
        index = 0;
    }
    (bucket, index)
}

/// A read-only OCCT `DataMap::Iterator`; `let mut it = map.cursor(); while it.more() { it.next(); }`.
pub struct DataMapIterator<'a, K, V> {
    items: DataMapItems<'a, K, V>,
}

/// A mutable OCCT `DataMap::Iterator` whose `change_value` edits items in place.
pub struct DataMapIteratorMut<'a, K, V> {
    buckets: &'a mut [Bucket<K, V>],
    bucket: usize,
    index: usize,
}

impl<K: Debug, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// `Iterator(const NCollection_DataMap&)`: a cursor on the first binding in bucket order.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(1, 10);
    /// let it = map.cursor();
    /// assert_eq!((it.key().unwrap(), it.value().unwrap()), (&1, &10));
    /// ```
    pub fn cursor(&self) -> DataMapIterator<'_, K, V> {
        DataMapIterator {
            items: self.items(),
        }
    }

    /// `Iterator(NCollection_DataMap&)`: a cursor that may also change items.
    ///
    /// ```
    /// # use rustycad_tkernel::ncollection::DataMap;
    /// let mut map = DataMap::<i32, i32>::default();
    /// map.bind(1, 10);
    /// *map.cursor_mut().change_value().unwrap() = 5;
    /// assert_eq!(map.seek(&1), Some(&5));
    /// ```
    pub fn cursor_mut(&mut self) -> DataMapIteratorMut<'_, K, V> {
        let (bucket, index) = settle(&self.buckets, 0, 0);
        DataMapIteratorMut {
            buckets: &mut self.buckets,
            bucket,
            index,
        }
    }
}

impl<'a, K, V> DataMapIterator<'a, K, V> {
    /// `More`: true while the cursor is on a binding.
    pub fn more(&self) -> bool {
        self.items.current().is_some()
    }
    /// `Next`: moves to the following binding in bucket order.
    pub fn next(&mut self) {
        self.items.next();
    }
    /// `Value`: the current item, or Standard_NoSuchObject past the end.
    pub fn value(&self) -> Result<&'a V, Failure> {
        Ok(&self.entry("NCollection_DataMap::Iterator::Value")?.value)
    }
    /// `Key`: the current key, or Standard_NoSuchObject past the end.
    pub fn key(&self) -> Result<&'a K, Failure> {
        Ok(&self.entry("NCollection_DataMap::Iterator::Key")?.key)
    }
    /// `Initialize(const NCollection_DataMap&)`: restarts on another map.
    pub fn initialize<H: HashPolicy<K>>(&mut self, map: &'a DataMap<K, V, H>)
    where
        K: Debug,
    {
        *self = map.cursor();
    }
    /// `Reset`: restarts on the map the cursor was created for (a default cursor stays empty).
    pub fn reset(&mut self) {
        (self.items.bucket, self.items.index) = settle(self.items.buckets, 0, 0);
    }
    /// `IsEqual`: both cursors are on the same binding, or both are past the end.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.items == other.items
    }
    fn entry(&self, method: &str) -> Result<&'a Entry<K, V>, Failure> {
        let entry = self
            .items
            .buckets
            .get(self.items.bucket)
            .and_then(|bucket| bucket.get(self.items.index));
        entry
            .map(|node| &**node)
            .ok_or_else(|| exhausted_iterator(method))
    }
}

/// `Iterator()`: a cursor attached to no map, whose `more()` is false.
impl<K, V> Default for DataMapIterator<'_, K, V> {
    fn default() -> Self {
        Self {
            items: DataMapItems {
                buckets: &[],
                bucket: 0,
                index: 0,
            },
        }
    }
}

impl<'a, K, V> DataMapIteratorMut<'a, K, V> {
    /// `More`: true while the cursor is on a binding.
    pub fn more(&self) -> bool {
        self.bucket < self.buckets.len()
    }
    /// `Next`: moves to the following binding in bucket order.
    pub fn next(&mut self) {
        if self.more() {
            (self.bucket, self.index) = settle(self.buckets, self.bucket, self.index + 1);
        }
    }
    /// `Value`: the current item, or Standard_NoSuchObject past the end.
    pub fn value(&self) -> Result<&V, Failure> {
        Ok(&self.entry("NCollection_DataMap::Iterator::Value")?.value)
    }
    /// `ChangeValue`: the current item for modification, or Standard_NoSuchObject past the end.
    pub fn change_value(&mut self) -> Result<&mut V, Failure> {
        let entry = self
            .buckets
            .get_mut(self.bucket)
            .and_then(|bucket| bucket.get_mut(self.index));
        let entry = entry
            .ok_or_else(|| exhausted_iterator("NCollection_DataMap::Iterator::ChangeValue"))?;
        Ok(&mut entry.value)
    }
    /// `Key`: the current key, or Standard_NoSuchObject past the end.
    pub fn key(&self) -> Result<&K, Failure> {
        Ok(&self.entry("NCollection_DataMap::Iterator::Key")?.key)
    }
    /// `Initialize(NCollection_DataMap&)`: restarts on another map.
    pub fn initialize<H: HashPolicy<K>>(&mut self, map: &'a mut DataMap<K, V, H>)
    where
        K: Debug,
    {
        *self = map.cursor_mut();
    }
    /// `Reset`: restarts on the map the cursor was created for.
    pub fn reset(&mut self) {
        (self.bucket, self.index) = settle(self.buckets, 0, 0);
    }
    fn entry(&self, method: &str) -> Result<&Entry<K, V>, Failure> {
        let entry = self
            .buckets
            .get(self.bucket)
            .and_then(|bucket| bucket.get(self.index));
        entry
            .map(|node| &**node)
            .ok_or_else(|| exhausted_iterator(method))
    }
}
