//! Native nullable value pointers, without dereferencing raw addresses.
use super::{Debug, FlatDataMap, HashPolicy};
impl<K: Debug, V, H: HashPolicy<K>> FlatDataMap<K, V, H> {
    /// Returns the value address or null for an unbound key; `map.seek_ptr(&1).is_null()`.
    pub fn seek_ptr(&self, key: &K) -> *const V {
        self.seek(key).map_or(std::ptr::null(), std::ptr::from_ref)
    }
    /// Returns a writable value address or null for an unbound key; `map.change_seek_ptr(&1)`.
    pub fn change_seek_ptr(&mut self, key: &K) -> *mut V {
        self.change_seek(key)
            .map_or(std::ptr::null_mut(), std::ptr::from_mut)
    }
}
