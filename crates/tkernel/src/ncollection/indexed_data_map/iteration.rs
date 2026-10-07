//! Keys and values in dense index order: the slot vectors are the native index table.
use super::{
    Debug, HashPolicy, IndexedDataItems, IndexedDataMap, IndexedDataValues, IndexedDataValuesMut,
};
impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Borrows values in dense index order; `map.iter().min()`.
    pub fn iter(&self) -> IndexedDataValues<'_, V> {
        IndexedDataValues::at(self.storage.payload().values(), 0)
    }
    /// Returns the exhausted value iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> IndexedDataValues<'_, V> {
        IndexedDataValues::at(self.storage.payload().values(), self.len())
    }
    /// Borrows all values exclusively; `map.iter_mut().for_each(|value|*value+=1)`.
    pub fn iter_mut(&mut self) -> IndexedDataValuesMut<'_, V> {
        IndexedDataValuesMut {
            items: self.storage.payload_mut().split_mut().1.iter_mut(),
        }
    }
    /// Borrows key/value pairs in index order; `for (key,value) in map.items() {}`.
    pub fn items(&self) -> IndexedDataItems<'_, K, V> {
        IndexedDataItems {
            keys: IndexedDataValues::at(self.storage.payload().keys(), 0),
            values: self.iter(),
        }
    }
    /// Returns the exhausted pair iterator; `map.items()!=map.items_end()`.
    pub fn items_end(&self) -> IndexedDataItems<'_, K, V> {
        IndexedDataItems {
            keys: IndexedDataValues::at(self.storage.payload().keys(), self.len()),
            values: self.end(),
        }
    }
    /// Borrows values exclusively while keeping keys immutable; `for (_,value) in map.items_mut() {}`.
    pub fn items_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        let (keys, values) = self.storage.payload_mut().split_mut();
        keys.iter().zip(values.iter_mut())
    }
    /// Borrows keys and values paired with one-based indices; `for (key,value,index) in map.indexed_items() {}`.
    pub fn indexed_items(&self) -> impl Iterator<Item = (&K, &V, usize)> {
        self.items()
            .enumerate()
            .map(|(index, (key, value))| (key, value, index + 1))
    }
}
impl<'a, V> IndexedDataValues<'a, V> {
    fn at(items: &'a [V], position: usize) -> Self {
        Self { items, position }
    }
}
impl<'a, V> Iterator for IndexedDataValues<'a, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.items.get(self.position)?;
        self.position += 1;
        Some(item)
    }
}
impl<V> Clone for IndexedDataValues<'_, V> {
    fn clone(&self) -> Self {
        Self::at(self.items, self.position)
    }
}
impl<V> PartialEq for IndexedDataValues<'_, V> {
    fn eq(&self, other: &Self) -> bool {
        (self.position >= self.items.len() && other.position >= other.items.len())
            || (self.items.as_ptr() == other.items.as_ptr() && self.position == other.position)
    }
}
impl<V> Eq for IndexedDataValues<'_, V> {}
impl<'a, V> Iterator for IndexedDataValuesMut<'a, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
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
