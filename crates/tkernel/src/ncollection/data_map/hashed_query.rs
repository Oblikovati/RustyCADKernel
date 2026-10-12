//! Borrowed specialization queries use the actual bucket chain without constructing an owned key.
use super::{BaseMap, DataMap, Debug, HashPolicy};
impl<K: Debug, V, H: HashPolicy<K>> DataMap<K, V, H> {
    /// Borrows a matching entry from an already computed native hash.
    /// Example: `map.find_hashed(7, |key| *key == 7)`.
    pub fn find_hashed(
        &self,
        hash: usize,
        mut accepts: impl FnMut(&K) -> bool,
    ) -> Option<(&K, &V)> {
        if self.is_empty() {
            return None;
        }
        let bucket = hash % self.storage.nb_buckets() + 1;
        let slot = self
            .storage
            .chains()
            .find(bucket, |slot| accepts(self.storage.payload().key(slot)))?;
        Some(self.storage.payload().entry(slot))
    }
}
#[cfg(test)]
mod tests {
    use super::DataMap;
    #[test]
    fn hashed_query_lends_actual_entries_and_skips_unrelated_buckets() {
        let mut map = DataMap::<i32, i32>::default();
        assert!(map.find_hashed(7, reject_unexpected_query).is_none());
        map.bind(7, 70);
        map.bind(108, 80);
        let mut compared = Vec::new();
        let found = map
            .find_hashed(7, |key| {
                compared.push(*key);
                *key == 7
            })
            .unwrap();
        assert_eq!(compared, [108, 7]);
        assert!(std::ptr::eq(found.1, map.find(&7).unwrap()));
        assert!(map.find_hashed(8, reject_unexpected_query).is_none());
    }
    fn reject_unexpected_query(key: &i32) -> bool {
        // Both empty maps and unrelated empty buckets must avoid key comparisons entirely.
        panic!("hashed query key={key}; expected no comparisons in an empty bucket")
    }
}
