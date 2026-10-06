//! Raw pointer results preserve native null-on-miss query behavior.
use super::{FlatMap, HashPolicy};

impl<K, H: HashPolicy<K>> FlatMap<K, H> {
    /// Returns a stored key address or native null when missing; `map.seek_ptr(&42).is_null()`.
    pub fn seek_ptr(&self, key: &K) -> *const K {
        self.seek(key).map_or(std::ptr::null(), std::ptr::from_ref)
    }
    /// Returns writable key storage or null; keep hash/equality identity intact; `map.change_seek_ptr(&42)`.
    pub fn change_seek_ptr(&mut self, key: &K) -> *mut K {
        self.change_seek(key)
            .map_or(std::ptr::null_mut(), std::ptr::from_mut)
    }
}
