//! Native `addImpl`/`emplaceImpl`: resize when `Resizable()`, hash once, then replace on a hit or
//! construct the value before the key and append on a miss.
use super::OrderedTable;
use crate::ncollection::HashPolicy;

/// An owned or a borrowed key: the borrowed form is cloned only when a new entry is made.
trait KeyArgument<K> {
    fn lookup_key(&self) -> &K;
    fn construct_key(self) -> K;
}
struct OwnedKey<K>(K);
struct BorrowedKey<'a, K>(&'a K);
impl<K> KeyArgument<K> for OwnedKey<K> {
    fn lookup_key(&self) -> &K {
        &self.0
    }
    fn construct_key(self) -> K {
        self.0
    }
}
impl<K: Clone> KeyArgument<K> for BorrowedKey<'_, K> {
    fn lookup_key(&self) -> &K {
        self.0
    }
    fn construct_key(self) -> K {
        self.0.clone()
    }
}

/// What a hit replaces: the set's `Emplace` reconstructs the key, the data map's `Bind` the item.
#[derive(Clone, Copy)]
struct Replace {
    key: bool,
    value: bool,
}

impl<K, V, H: HashPolicy<K>> OrderedTable<K, V, H> {
    /// Adds or updates `key`; returns whether it was new and its slot.
    pub fn place(
        &mut self,
        key: K,
        construct: impl FnOnce() -> V,
        replace_key: bool,
        replace_value: bool,
    ) -> (bool, usize) {
        self.prepare_insertion();
        let replace = Replace {
            key: replace_key,
            value: replace_value,
        };
        self.place_prepared(OwnedKey(key), construct, replace)
    }
    fn prepare_insertion(&mut self) {
        if self.len() == 0 || self.len() > self.bucket_count() {
            self.resize(self.len());
        }
    }
    fn place_prepared<I: KeyArgument<K>>(
        &mut self,
        key: I,
        construct: impl FnOnce() -> V,
        replace: Replace,
    ) -> (bool, usize) {
        let (bucket, slot) = self
            .storage
            .lookup_with_bucket(key.lookup_key(), &self.hasher);
        let Some(slot) = slot else {
            let slot = self
                .storage
                .insert(bucket, || key.construct_key(), construct);
            self.append_order(slot);
            return (true, slot);
        };
        self.replace_entry(slot, key, construct, replace);
        (false, slot)
    }
    fn replace_entry<I: KeyArgument<K>>(
        &mut self,
        slot: usize,
        key: I,
        construct: impl FnOnce() -> V,
        replace: Replace,
    ) {
        let (stored_key, stored_value) = self.storage.payload_mut().pair_mut(slot);
        if replace.key {
            *stored_key = key.construct_key();
        }
        if replace.value {
            *stored_value = construct();
        }
    }
}

impl<K: Clone, V, H: HashPolicy<K>> OrderedTable<K, V, H> {
    /// Adds a borrowed key, cloning it only for a new entry; a hit replaces only the value.
    pub fn place_key_ref(
        &mut self,
        key: &K,
        construct: impl FnOnce() -> V,
        replace_value: bool,
    ) -> (bool, usize) {
        self.prepare_insertion();
        let replace = Replace {
            key: false,
            value: replace_value,
        };
        self.place_prepared(BorrowedKey(key), construct, replace)
    }
}

impl<K: Clone, V: Clone, H: HashPolicy<K>> OrderedTable<K, V, H> {
    /// OrderedDataMap copies the item before its key and never copies keys on hits.
    pub fn place_borrowed(&mut self, key: &K, value: &V, replace: bool) -> (bool, usize) {
        // Native emplaceImpl copy-constructs a temporary, then move-assigns it on hits.
        self.place_key_ref(key, || value.clone(), replace)
    }
}

impl<K, H: HashPolicy<K>> OrderedTable<K, (), H> {
    /// Set `Emplace`/`TryEmplace` with a deferred key: OCCT 3d097a0 emplaceImpl resizes before
    /// constructing the temporary, even for a duplicate Try.
    pub fn place_constructed_key(
        &mut self,
        construct: impl FnOnce() -> K,
        replace: bool,
    ) -> (bool, usize) {
        self.prepare_insertion();
        let key = construct();
        let replace = Replace {
            key: replace,
            value: false,
        };
        self.place_prepared(OwnedKey(key), || (), replace)
    }
}
