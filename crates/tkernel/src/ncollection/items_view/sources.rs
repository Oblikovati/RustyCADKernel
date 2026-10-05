//! The maps that expose native `Items()`/`IndexedItems()` views, with their extractors.
use super::{ItemsSource, ItemsSourceMut, KeyIndexItems, KeyValueIndexItems, KeyValueItems};
use crate::ncollection::{
    DataMap, FlatDataMap, HashPolicy, IndexedDataMap, IndexedMap, OrderedDataMap,
};
use std::fmt::Debug;

/// Implements the key/value extractor (read and write) for a map with `items`/`items_mut`.
macro_rules! key_value_source {
    ($map:ident) => {
        impl<K: Debug, V, H: HashPolicy<K>> ItemsSource<KeyValueItems> for $map<K, V, H> {
            type Item<'a>
                = (&'a K, &'a V)
            where
                Self: 'a;
            fn items_begin(&self) -> impl Iterator<Item = Self::Item<'_>> {
                self.items()
            }
            fn items_size(&self) -> usize {
                self.len()
            }
            fn items_is_empty(&self) -> bool {
                self.is_empty()
            }
        }
        impl<K: Debug, V, H: HashPolicy<K>> ItemsSourceMut<KeyValueItems> for $map<K, V, H> {
            type ItemMut<'a>
                = (&'a K, &'a mut V)
            where
                Self: 'a;
            fn items_begin_mut(&mut self) -> impl Iterator<Item = Self::ItemMut<'_>> {
                self.items_mut()
            }
        }
    };
}

key_value_source!(DataMap);
key_value_source!(IndexedDataMap);
key_value_source!(FlatDataMap);
key_value_source!(OrderedDataMap);

impl<K: Debug, V, H: HashPolicy<K>> ItemsSource<KeyValueIndexItems> for IndexedDataMap<K, V, H> {
    type Item<'a>
        = (&'a K, &'a V, usize)
    where
        Self: 'a;
    fn items_begin(&self) -> impl Iterator<Item = Self::Item<'_>> {
        self.indexed_items()
    }
    fn items_size(&self) -> usize {
        self.len()
    }
    fn items_is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<K: Debug, H: HashPolicy<K>> ItemsSource<KeyIndexItems> for IndexedMap<K, H> {
    type Item<'a>
        = (&'a K, usize)
    where
        Self: 'a;
    fn items_begin(&self) -> impl Iterator<Item = Self::Item<'_>> {
        self.indexed_items()
    }
    fn items_size(&self) -> usize {
        self.len()
    }
    fn items_is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<K: Debug, V, H: HashPolicy<K>> ItemsSourceMut<KeyValueIndexItems> for IndexedDataMap<K, V, H> {
    type ItemMut<'a>
        = (&'a K, &'a mut V, usize)
    where
        Self: 'a;

    /// Preserves writable values and one-based indices; `view.begin()`.
    fn items_begin_mut(&mut self) -> impl Iterator<Item = Self::ItemMut<'_>> {
        self.indexed_items_mut()
    }
}
