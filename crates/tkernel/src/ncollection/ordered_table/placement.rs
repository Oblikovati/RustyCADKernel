//! Ownership-specific key inputs share native allocation, hashing and construction phases.
use super::{HashPolicy, OrderedNode, OrderedTable};

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
impl<K, V, H: HashPolicy<K>> OrderedTable<K, V, H> {
    pub fn place(
        &mut self,
        key: K,
        construct: impl FnOnce() -> V,
        replace_key: bool,
        replace_value: bool,
    ) -> (bool, usize) {
        self.place_argument(OwnedKey(key), construct, replace_key, replace_value)
    }
    fn place_argument<I: KeyArgument<K>>(
        &mut self,
        key: I,
        construct: impl FnOnce() -> V,
        replace_key: bool,
        replace_value: bool,
    ) -> (bool, usize) {
        if self.count == 0 || self.count > self.bucket_count {
            self.resize(self.count);
        }
        let hash = self.hasher.hash(key.lookup_key()) % self.bucket_count + 1;
        if let Some(slot) = self.lookup_at_hash(key.lookup_key(), hash) {
            self.replace_argument(slot, key, construct, replace_key, replace_value);
            return (false, slot);
        }
        (true, self.append_argument(key, construct, hash))
    }
    fn replace_argument<I: KeyArgument<K>>(
        &mut self,
        slot: usize,
        key: I,
        construct: impl FnOnce() -> V,
        replace_key: bool,
        replace_value: bool,
    ) {
        let node = self.slots[slot].as_mut().unwrap();
        if replace_key {
            node.key = key.construct_key();
        }
        if replace_value {
            node.value = construct();
        }
    }
    fn append_argument<I: KeyArgument<K>>(
        &mut self,
        key: I,
        construct: impl FnOnce() -> V,
        hash: usize,
    ) -> usize {
        let node = self.resource.allocate_value_with(|| {
            // OCCT 3d097a03 constructs the base value before the derived key.
            let value = construct();
            OrderedNode {
                key: key.construct_key(),
                value,
                previous: self.last,
                next: None,
            }
        });
        self.append_allocated(node, hash)
    }
}
impl<K: Clone, V, H: HashPolicy<K>> OrderedTable<K, V, H> {
    pub fn place_key_ref(
        &mut self,
        key: &K,
        construct: impl FnOnce() -> V,
        replace: bool,
    ) -> (bool, usize) {
        self.place_argument(BorrowedKey(key), construct, false, replace)
    }
}
