//! OCCT assignment identity and storage exchange contracts.
use super::{Debug, HashPolicy, IndexedMap};

impl<K: Debug, H: HashPolicy<K>> IndexedMap<K, H> {
    /// Exchanges both maps as native move assignment does; `target.move_from(&mut source)`.
    pub fn move_from(&mut self, source: &mut Self) -> &mut Self {
        self.exchange(source);
        self
    }

    /// Preserves self-assignment without creating aliased references; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}
