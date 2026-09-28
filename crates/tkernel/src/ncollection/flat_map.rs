// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Contiguous Robin Hood hash keys preserving native growth and displacement behavior.
use super::flat_table::{FlatItems, FlatTable};
use super::{DefaultHashPolicy, HashPolicy};
/// An open-addressed hash set; `FlatMap::<i32>::default().add(42)`.
pub struct FlatMap<K, H = DefaultHashPolicy> {
    table: FlatTable<K, (), H>,
}
/// Shared keys in contiguous slot order; `map.iter()`.
pub struct FlatMapIter<'a, K> {
    items: FlatItems<'a, K, ()>,
}
impl<K, H: HashPolicy<K>> FlatMap<K, H> {
    /// Injects hash/equality policy and an element-capacity hint; `FlatMap::with_hasher(policy,10)`.
    pub fn with_hasher(hasher: H, hint: usize) -> Self {
        Self {
            table: FlatTable::new(hasher, hint),
        }
    }
    /// Returns unique key count; `map.len()==3`.
    pub fn len(&self) -> usize {
        self.table.count
    }
    /// Returns OCCT's key extent; `map.extent()==3`.
    pub fn extent(&self) -> usize {
        self.len()
    }
    /// Checks whether there are no keys; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Returns the number of contiguous slots; `map.capacity()`.
    pub fn capacity(&self) -> usize {
        self.table.slots.len()
    }
    /// Borrows the retained policy; `map.hasher().salt`.
    pub fn hasher(&self) -> &H {
        &self.table.hasher
    }
    /// Reserves room for the requested key count at OCCT's 13/16 load limit; `map.reserve(100)`.
    pub fn reserve(&mut self, count: usize) {
        self.table.reserve(count);
    }
    /// Tests membership through Robin Hood probing; `map.contains(&42)`.
    pub fn contains(&self, key: &K) -> bool {
        self.table.lookup(key).is_some()
    }
    /// Borrows the stored equivalent key, OCCT's optional reference; `map.contained(&42)`.
    pub fn contained(&self, key: &K) -> Option<&K> {
        self.seek(key)
    }
    /// Optionally borrows the stored key; `map.seek(&42)`.
    pub fn seek(&self, key: &K) -> Option<&K> {
        Some(&self.table.slots[self.table.lookup(key)?].as_ref()?.key)
    }
    /// Borrows a key exclusively; callers must preserve its hash/equality identity; `map.change_seek(&42)`.
    pub fn change_seek(&mut self, key: &K) -> Option<&mut K> {
        let index = self.table.lookup(key)?;
        Some(&mut self.table.slots[index].as_mut()?.key)
    }
    /// Inserts without replacing an equivalent stored key; `map.add(42)`.
    pub fn add(&mut self, key: K) -> bool {
        self.table.insert(key, (), false, false).is_new
    }
    /// Returns the inserted key even after Robin Hood displacement; `map.added(42)`.
    pub fn added(&mut self, key: K) -> &K {
        let result = self.table.insert(key, (), false, false);
        &self.table.slots[result.original].as_ref().unwrap().key
    }
    /// Reconstructs an equivalent key and reports insertion; `map.emplace(42)`.
    pub fn emplace(&mut self, key: K) -> bool {
        self.table.insert(key, (), true, false).is_new
    }
    /// Preserves OCCT's final-slot return after displacement; `map.emplaced(42)`.
    pub fn emplaced(&mut self, key: K) -> &K {
        let result = self.table.insert(key, (), true, false);
        &self.table.slots[result.last].as_ref().unwrap().key
    }
    /// Inserts only when absent; `map.try_emplace(42)`.
    pub fn try_emplace(&mut self, key: K) -> bool {
        self.add(key)
    }
    /// Preserves OCCT's final-slot reference without replacing duplicates; `map.try_emplaced(42)`.
    pub fn try_emplaced(&mut self, key: K) -> &K {
        let result = self.table.insert(key, (), false, false);
        &self.table.slots[result.last].as_ref().unwrap().key
    }
    /// Removes a key and shifts the probe cluster backward; `map.remove(&42)`.
    pub fn remove(&mut self, key: &K) -> bool {
        self.table.remove(key)
    }
    /// Drops keys while optionally retaining slot capacity; `map.clear(false)`.
    pub fn clear(&mut self, release: bool) {
        self.table.clear(release);
    }
    /// Swaps policies and storage; `map.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// Borrows keys in slot order; `for key in map.iter() {}`.
    pub fn iter(&self) -> FlatMapIter<'_, K> {
        FlatMapIter {
            items: self.table.items(),
        }
    }
    /// Returns an exhausted iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> FlatMapIter<'_, K> {
        FlatMapIter {
            items: FlatItems {
                slots: &self.table.slots,
                index: self.capacity(),
            },
        }
    }
}
impl<K, H: HashPolicy<K> + Default> FlatMap<K, H> {
    /// Selects capacity with a default policy; `FlatMap::<i32>::new(10)`.
    pub fn new(hint: usize) -> Self {
        Self::with_hasher(H::default(), hint)
    }
    /// Moves storage and leaves an empty source; `FlatMap::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// Native move assignment: releases this storage, then takes the source's and leaves
    /// the source empty with no slots; `map.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = Self::move_from(source);
    }
}
impl<K, H: HashPolicy<K> + Default> Default for FlatMap<K, H> {
    fn default() -> Self {
        Self::new(0)
    }
}
impl<K: Clone, H: Clone> Clone for FlatMap<K, H> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
        }
    }
}
impl<'a, K> Iterator for FlatMapIter<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(|(key, ())| key)
    }
}
impl<K> Clone for FlatMapIter<'_, K> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
        }
    }
}
/// The native default `Iterator()`: exhausted, equal to every end iterator.
impl<K> Default for FlatMapIter<'_, K> {
    fn default() -> Self {
        Self {
            items: FlatItems::default(),
        }
    }
}
impl<K> PartialEq for FlatMapIter<'_, K> {
    fn eq(&self, other: &Self) -> bool {
        self.items == other.items
    }
}
impl<K> Eq for FlatMapIter<'_, K> {}
impl<'a, K, H: HashPolicy<K>> IntoIterator for &'a FlatMap<K, H> {
    type Item = &'a K;
    type IntoIter = FlatMapIter<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
