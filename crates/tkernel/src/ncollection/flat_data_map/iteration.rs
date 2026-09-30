use super::{
    Debug, FlatDataItems, FlatDataMap, FlatDataValues, FlatDataValuesMut, FlatItems, HashPolicy,
};
impl<K: Debug, V, H: HashPolicy<K>> FlatDataMap<K, V, H> {
    /// Borrows values in slot order; `map.iter().min()`.
    pub fn iter(&self) -> FlatDataValues<'_, K, V> {
        FlatDataValues {
            inner: self.table.items(),
        }
    }
    /// Returns an exhausted value iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> FlatDataValues<'_, K, V> {
        FlatDataValues {
            inner: self.end_items(),
        }
    }
    /// Borrows values exclusively; `map.iter_mut().for_each(|value|*value+=1)`.
    pub fn iter_mut(&mut self) -> FlatDataValuesMut<'_, K, V> {
        FlatDataValuesMut {
            slots: self.table.slots.iter_mut(),
        }
    }
    /// Borrows key/value pairs in slot order; `for (key,value) in map.items() {}`.
    pub fn items(&self) -> FlatDataItems<'_, K, V> {
        FlatDataItems {
            inner: self.table.items(),
        }
    }
    /// Returns an exhausted pair iterator; `map.items()!=map.items_end()`.
    pub fn items_end(&self) -> FlatDataItems<'_, K, V> {
        FlatDataItems {
            inner: self.end_items(),
        }
    }
    /// Borrows values exclusively while keeping keys immutable; `for (_,value) in map.items_mut() {}`.
    pub fn items_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.table
            .slots
            .iter_mut()
            .filter_map(Option::as_mut)
            .map(|slot| (&slot.key, &mut slot.value))
    }
    fn end_items(&self) -> FlatItems<'_, K, V> {
        FlatItems {
            slots: &self.table.slots,
            index: self.capacity(),
        }
    }
}
impl<'a, K, V> Iterator for FlatDataItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
impl<K, V> Clone for FlatDataItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// The native default `Iterator()`: exhausted, equal to every end iterator.
impl<K, V> Default for FlatDataItems<'_, K, V> {
    fn default() -> Self {
        Self {
            inner: FlatItems::default(),
        }
    }
}
impl<K, V> PartialEq for FlatDataItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<K, V> Eq for FlatDataItems<'_, K, V> {}
impl<'a, K, V> Iterator for FlatDataValues<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }
}
impl<K, V> Clone for FlatDataValues<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// The exhausted value iterator a default native `const_iterator` denotes.
impl<K, V> Default for FlatDataValues<'_, K, V> {
    fn default() -> Self {
        Self {
            inner: FlatItems::default(),
        }
    }
}
impl<K, V> PartialEq for FlatDataValues<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<K, V> Eq for FlatDataValues<'_, K, V> {}
impl<'a, K, V> Iterator for FlatDataValuesMut<'a, K, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<Self::Item> {
        self.slots
            .find_map(Option::as_mut)
            .map(|slot| &mut slot.value)
    }
}
impl<'a, K: Debug, V, H: HashPolicy<K>> IntoIterator for &'a FlatDataMap<K, V, H> {
    type Item = &'a V;
    type IntoIter = FlatDataValues<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, K: Debug, V, H: HashPolicy<K>> IntoIterator for &'a mut FlatDataMap<K, V, H> {
    type Item = &'a mut V;
    type IntoIter = FlatDataValuesMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
