//! OrderedDataMap copies the item before its key and never copies keys on hits.
use super::{HashPolicy, OrderedNode, OrderedTable};

impl<K: Clone, V: Clone, H: HashPolicy<K>> OrderedTable<K, V, H> {
    pub fn place_borrowed(&mut self, key: &K, value: &V, replace: bool) -> (bool, usize) {
        if self.count == 0 || self.count > self.bucket_count {
            self.resize(self.count);
        }
        let hash = self.hasher.hash(key) % self.bucket_count + 1;
        if let Some(slot) = self.lookup_at_hash(key, hash) {
            if replace {
                // Native emplaceImpl copy-constructs a temporary, then move-assigns it.
                self.slots[slot].as_mut().unwrap().value = value.clone();
            }
            return (false, slot);
        }
        (true, self.append_borrowed(key, value, hash))
    }

    fn append_borrowed(&mut self, key: &K, value: &V, hash: usize) -> usize {
        let node = self.resource.allocate_value_with(|| {
            let copied_value = value.clone();
            let copied_key = key.clone();
            OrderedNode {
                key: copied_key,
                value: copied_value,
                previous: self.last,
                next: None,
            }
        });
        self.append_allocated(node, hash)
    }
}
