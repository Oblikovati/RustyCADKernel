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

use super::{Debug, HashPolicy, IndexedMap, IndexedMapItems, IndexedMapIter};
impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Traverses keys in dense index order; `map.iter().min()`.
    pub fn iter(&self) -> IndexedMapIter<'_, K> {
        IndexedMapIter {
            nodes: &self.nodes,
            position: 0,
        }
    }
    /// Returns an exhausted key iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> IndexedMapIter<'_, K> {
        IndexedMapIter {
            nodes: &self.nodes,
            position: self.len(),
        }
    }
    /// Traverses keys paired with one-based indices; `for (key,index) in map.indexed_items() {}`.
    pub fn indexed_items(&self) -> IndexedMapItems<'_, K> {
        IndexedMapItems { inner: self.iter() }
    }
    /// Returns the exhausted indexed range; `map.indexed_items()!=map.indexed_end()`.
    pub fn indexed_end(&self) -> IndexedMapItems<'_, K> {
        IndexedMapItems { inner: self.end() }
    }
}
impl<'a, K> Iterator for IndexedMapIter<'a, K> {
    type Item = &'a K;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.nodes.get(self.position)?;
        self.position += 1;
        Some(&node.key)
    }
}
impl<K> Clone for IndexedMapIter<'_, K> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes,
            position: self.position,
        }
    }
}
impl<K> PartialEq for IndexedMapIter<'_, K> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes.as_ptr() == other.nodes.as_ptr() && self.position == other.position
    }
}
impl<K> Eq for IndexedMapIter<'_, K> {}
impl<'a, K> Iterator for IndexedMapItems<'a, K> {
    type Item = (&'a K, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.inner.position + 1;
        self.inner.next().map(|key| (key, index))
    }
}
impl<K> Clone for IndexedMapItems<'_, K> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<K> PartialEq for IndexedMapItems<'_, K> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<K> Eq for IndexedMapItems<'_, K> {}
impl<'a, K: Debug, H: HashPolicy<K>> IntoIterator for &'a IndexedMap<K, H> {
    type Item = &'a K;
    type IntoIter = IndexedMapIter<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
