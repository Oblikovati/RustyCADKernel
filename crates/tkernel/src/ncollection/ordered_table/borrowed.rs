//! OrderedDataMap copies the item before its key and never copies keys on hits.
use super::{HashPolicy, OrderedTable};

impl<K: Clone, V: Clone, H: HashPolicy<K>> OrderedTable<K, V, H> {
    pub fn place_borrowed(&mut self, key: &K, value: &V, replace: bool) -> (bool, usize) {
        // Native emplaceImpl copy-constructs a temporary, then move-assigns it on hits.
        self.place_key_ref(key, || value.clone(), replace)
    }
}
