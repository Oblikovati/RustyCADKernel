//! Borrowed const-reference overloads retain native hash/key/item copy ordering.
use super::{Debug, FlatDataMap, HashPolicy};

macro_rules! borrowed_pair_binding {
    ($boolean:ident, $reference:ident, $replace:literal) => {
        impl<K: Clone + Debug, V: Clone, H: HashPolicy<K>> FlatDataMap<K, V, H> {
            /// Copies key then item after hashing, even on hits; `map.bind_ref(&key, &item)`.
            pub fn $boolean(&mut self, key: &K, item: &V) -> bool {
                self.table.insert_copied_pair(key, item, $replace).is_new
            }
            /// Returns stored item after copying borrowed arguments; `map.bound_ref(&key, &item)`.
            pub fn $reference(&mut self, key: &K, item: &V) -> &mut V {
                let result = self.table.insert_copied_pair(key, item, $replace);
                self.value_at(result.original)
            }
        }
    };
}
macro_rules! borrowed_key_binding {
    ($boolean:ident, $reference:ident, $replace:literal) => {
        impl<K: Clone + Debug, V, H: HashPolicy<K>> FlatDataMap<K, V, H> {
            /// Copies a borrowed key after hashing and takes the item; `map.bind_ref_key(&key, item)`.
            pub fn $boolean(&mut self, key: &K, item: V) -> bool {
                self.table.insert_copied_key(key, item, $replace).is_new
            }
            /// Returns the stored item after a borrowed key copy; `map.bound_ref_key(&key, item)`.
            pub fn $reference(&mut self, key: &K, item: V) -> &mut V {
                let result = self.table.insert_copied_key(key, item, $replace);
                self.value_at(result.original)
            }
        }
    };
}
macro_rules! borrowed_value_binding {
    ($boolean:ident, $reference:ident, $replace:literal) => {
        impl<K: Debug, V: Clone, H: HashPolicy<K>> FlatDataMap<K, V, H> {
            /// Takes the key and copies a borrowed item after hashing; `map.bind_ref_value(key, &item)`.
            pub fn $boolean(&mut self, key: K, item: &V) -> bool {
                self.table.insert_copied_value(key, item, $replace).is_new
            }
            /// Returns the stored item after a borrowed value copy; `map.bound_ref_value(key, &item)`.
            pub fn $reference(&mut self, key: K, item: &V) -> &mut V {
                let result = self.table.insert_copied_value(key, item, $replace);
                self.value_at(result.original)
            }
        }
    };
}
borrowed_pair_binding!(bind_ref, bound_ref, true);
borrowed_pair_binding!(try_bind_ref, try_bound_ref, false);
borrowed_key_binding!(bind_ref_key, bound_ref_key, true);
borrowed_key_binding!(try_bind_ref_key, try_bound_ref_key, false);
borrowed_value_binding!(bind_ref_value, bound_ref_value, true);
borrowed_value_binding!(try_bind_ref_value, try_bound_ref_value, false);
