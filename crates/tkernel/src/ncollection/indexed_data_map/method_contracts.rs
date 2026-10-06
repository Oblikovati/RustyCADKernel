//! Borrowed binding and returning assignment contracts.
use super::{Debug, HashPolicy, IndexedDataMap};

impl<K: Debug, V, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Exchanges entries, allocators and policies like native move assignment; `target.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) -> &mut Self {
        self.exchange(source);
        self
    }

    /// Keeps self-assignment safe without aliasing mutable references; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}

impl<K: Debug + Clone, V: Clone, H: HashPolicy<K>> IndexedDataMap<K, V, H> {
    /// Copies a binding, assigning only the item on a hit; `map.bind_ref(&1, &42)`.
    pub fn bind_ref(&mut self, key: &K, value: &V) -> bool {
        let count = self.len();
        self.place_borrowed(key, value, true) > count
    }

    /// Copies a binding and borrows its stored item; `*map.bound_ref(&1, &42) = 7`.
    pub fn bound_ref(&mut self, key: &K, value: &V) -> &mut V {
        let index = self.place_borrowed(key, value, true);
        self.storage.value_mut(index)
    }

    /// Copies arguments only if no binding exists; `map.try_bind_ref(&1, &42)`.
    pub fn try_bind_ref(&mut self, key: &K, value: &V) -> bool {
        let count = self.len();
        self.place_borrowed(key, value, false) > count
    }

    /// Borrows a stored item, copying arguments only on a miss; `map.try_bound_ref(&1, &42)`.
    pub fn try_bound_ref(&mut self, key: &K, value: &V) -> &mut V {
        let index = self.place_borrowed(key, value, false);
        self.storage.value_mut(index)
    }

    fn place_borrowed(&mut self, key: &K, value: &V, replace: bool) -> usize {
        self.prepare_borrowed_add();
        let (index, bucket) = self.storage.lookup(key, &self.hasher);
        if index == 0 {
            return self.insert_borrowed(key, value, bucket);
        }
        if replace {
            self.storage.value_mut(index).clone_from(value);
        }
        index
    }
}
