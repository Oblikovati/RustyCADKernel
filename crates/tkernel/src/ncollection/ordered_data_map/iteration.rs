use super::{
    HashPolicy, OrderedDataItems, OrderedDataItemsMut, OrderedDataMap, OrderedDataValues,
    OrderedDataValuesMut, OrderedItems,
};
impl<K, V, H: HashPolicy<K>> OrderedDataMap<K, V, H> {
    /// Borrows values in insertion order; `map.iter()`.
    pub fn iter(&self) -> OrderedDataValues<'_, K, V> {
        OrderedDataValues {
            inner: self.table.items(),
        }
    }
    /// Returns an exhausted value iterator; `map.iter()!=map.end()`.
    pub fn end(&self) -> OrderedDataValues<'_, K, V> {
        OrderedDataValues {
            inner: OrderedItems::default(),
        }
    }
    /// Borrows values exclusively in insertion order; `map.iter_mut()`.
    pub fn iter_mut(&mut self) -> OrderedDataValuesMut<'_, K, V> {
        OrderedDataValuesMut {
            inner: self.table.items_mut(),
        }
    }
    /// Borrows pairs in insertion order; `for (key,value) in map.items() {}`.
    pub fn items(&self) -> OrderedDataItems<'_, K, V> {
        OrderedDataItems {
            inner: self.table.items(),
        }
    }
    /// Returns an exhausted pair iterator; `map.items()!=map.items_end()`.
    pub fn items_end(&self) -> OrderedDataItems<'_, K, V> {
        OrderedDataItems {
            inner: OrderedItems::default(),
        }
    }
    /// Borrows immutable keys and exclusive values in insertion order; `map.items_mut()`.
    pub fn items_mut(&mut self) -> OrderedDataItemsMut<'_, K, V> {
        OrderedDataItemsMut {
            inner: self.table.items_mut(),
        }
    }
}
impl<'a, K, V> Iterator for OrderedDataItems<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
impl<K, V> Clone for OrderedDataItems<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<K, V> PartialEq for OrderedDataItems<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<K, V> Eq for OrderedDataItems<'_, K, V> {}
impl<'a, K, V> Iterator for OrderedDataItemsMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
impl<'a, K, V> Iterator for OrderedDataValues<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }
}
impl<K, V> Clone for OrderedDataValues<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<K, V> PartialEq for OrderedDataValues<'_, K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<K, V> Eq for OrderedDataValues<'_, K, V> {}
impl<'a, K, V> Iterator for OrderedDataValuesMut<'a, K, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }
}
impl<'a, K, V, H: HashPolicy<K>> IntoIterator for &'a OrderedDataMap<K, V, H> {
    type Item = &'a V;
    type IntoIter = OrderedDataValues<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, K, V, H: HashPolicy<K>> IntoIterator for &'a mut OrderedDataMap<K, V, H> {
    type Item = &'a mut V;
    type IntoIter = OrderedDataValuesMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
