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

use super::{
    Debug, HashPolicy, IndexedDataItems, IndexedDataMap, IndexedDataValues, IndexedDataValuesMut,
};
impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Borrows values in dense index order; `map.iter().min()`.
    pub fn iter(&self) -> IndexedDataValues<'_, V> {
        IndexedDataValues {
            nodes: &self.values,
            position: 0,
        }
    }
    /// Returns the exhausted value iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> IndexedDataValues<'_, V> {
        IndexedDataValues {
            nodes: &self.values,
            position: self.len(),
        }
    }
    /// Borrows all values exclusively; `map.iter_mut().for_each(|value|*value+=1)`.
    pub fn iter_mut(&mut self) -> IndexedDataValuesMut<'_, V> {
        IndexedDataValuesMut {
            nodes: self.values.iter_mut(),
        }
    }
    /// Borrows key/value pairs in index order; `for (key,value) in map.items() {}`.
    pub fn items(&self) -> IndexedDataItems<'_, K, V> {
        IndexedDataItems {
            keys: self.keys.iter(),
            values: self.iter(),
        }
    }
    /// Returns the exhausted pair iterator; `map.items()!=map.items_end()`.
    pub fn items_end(&self) -> IndexedDataItems<'_, K, V> {
        IndexedDataItems {
            keys: self.keys.end(),
            values: self.end(),
        }
    }
    /// Borrows values exclusively while keeping keys immutable; `for (_,value) in map.items_mut() {}`.
    pub fn items_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.keys
            .iter()
            .zip(self.values.iter_mut().map(|node| &mut node.value))
    }
    /// Borrows keys and values paired with one-based indices; `for (key,value,index) in map.indexed_items() {}`.
    pub fn indexed_items(&self) -> impl Iterator<Item = (&K, &V, usize)> {
        self.items()
            .enumerate()
            .map(|(index, (key, value))| (key, value, index + 1))
    }
}
impl<'a, V> Iterator for IndexedDataValues<'a, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        let node = self.nodes.get(self.position)?;
        self.position += 1;
        Some(&node.value)
    }
}
impl<V> Clone for IndexedDataValues<'_, V> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes,
            position: self.position,
        }
    }
}
impl<V> PartialEq for IndexedDataValues<'_, V> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes.as_ptr() == other.nodes.as_ptr() && self.position == other.position
    }
}
impl<V> Eq for IndexedDataValues<'_, V> {}
impl<'a, V> Iterator for IndexedDataValuesMut<'a, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<Self::Item> {
        self.nodes.next().map(|node| &mut node.value)
    }
}
impl<'a, K, V> Iterator for IndexedDataItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.keys.next().zip(self.values.next())
    }
}
impl<K, V> Clone for IndexedDataItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            keys: self.keys.clone(),
            values: self.values.clone(),
        }
    }
}
impl<K, V> PartialEq for IndexedDataItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.keys == other.keys && self.values == other.values
    }
}
impl<K, V> Eq for IndexedDataItems<'_, K, V> {}
impl<'a, K: Debug, V, H: HashPolicy<K>> IntoIterator for &'a IndexedDataMap<K, V, H> {
    type Item = &'a V;
    type IntoIter = IndexedDataValues<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, K: Debug, V, H: HashPolicy<K>> IntoIterator for &'a mut IndexedDataMap<K, V, H> {
    type Item = &'a mut V;
    type IntoIter = IndexedDataValuesMut<'a, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
