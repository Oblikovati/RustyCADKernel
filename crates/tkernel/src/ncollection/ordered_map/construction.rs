//! Deferred set-key construction preserves the native resize/construction/lookup phases.
use super::{HashPolicy, OrderedMap};
impl<K, H: HashPolicy<K>> OrderedMap<K, H> {
    /// Constructs after resize, replacing an equivalent key; `map.emplace_with(|| make_key())`.
    /// Key transfer uses Rust moves; custom C++ move-constructor effects require a separate policy.
    pub fn emplace_with(&mut self, construct: impl FnOnce() -> K) -> bool {
        self.table.place_constructed_key(construct, true).0
    }
    /// Constructs after resize and returns stable stored key space; `map.emplaced_with(|| 42)`.
    pub fn emplaced_with(&mut self, construct: impl FnOnce() -> K) -> &K {
        let (_, slot) = self.table.place_constructed_key(construct, true);
        self.table.key(slot)
    }
    /// Still constructs a temporary on a duplicate hit; `map.try_emplace_with(|| 42)`.
    /// An existing key is retained and the unused temporary is dropped before returning.
    pub fn try_emplace_with(&mut self, construct: impl FnOnce() -> K) -> bool {
        self.table.place_constructed_key(construct, false).0
    }
    /// Returns the existing key or transfers the constructed temporary; `map.try_emplaced_with(|| 42)`.
    pub fn try_emplaced_with(&mut self, construct: impl FnOnce() -> K) -> &K {
        let (_, slot) = self.table.place_constructed_key(construct, false);
        self.table.key(slot)
    }
}
