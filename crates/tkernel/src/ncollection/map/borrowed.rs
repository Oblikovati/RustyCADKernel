//! Borrowed insertion keeps native lookup and key-copy ordering.
use super::{Debug, HashPolicy, Map};

impl<K: Debug + Clone, H: HashPolicy<K>> Map<K, H> {
    /// Adds a borrowed key only when absent; lookup precedes allocation and copying.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Map;
    /// let mut keys = Map::<i32>::default();
    /// assert!(keys.add_ref(&7));
    /// assert!(!keys.add_ref(&7));
    /// ```
    pub fn add_ref(&mut self, key: &K) -> bool {
        self.bindings.place_key_ref(key).0
    }
    /// Returns a stable stored key, copying the borrowed key only on insertion.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::Map;
    /// let mut keys = Map::<i32>::default();
    /// assert_eq!(keys.added_ref(&7), &7);
    /// assert_eq!(keys.added_ref(&7), &7);
    /// ```
    pub fn added_ref(&mut self, key: &K) -> &K {
        self.bindings.place_key_ref(key).1
    }
}
