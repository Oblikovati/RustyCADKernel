//! Native aliases and self-aliased expressions represented without conflicting Rust borrows.
use super::{PackedInteger, PackedMap, PackedMapIter};
impl<T: PackedInteger> PackedMap<T> {
    /// Returns native unsigned Size; `assert_eq!(map.size(), map.len())`.
    pub fn size(&self) -> usize {
        self.len()
    }
    /// Returns native signed Length; `assert_eq!(map.length(), map.extent())`.
    pub fn length(&self) -> i32 {
        self.extent()
    }
    /// Returns the detached STL end iterator; `assert!(!map.end().more())`.
    pub fn end(&self) -> PackedMapIter<'_, T> {
        PackedMapIter::default()
    }
    /// Preserves native copy/move self-assignment; `map.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
    /// Preserves the unchanged self-union branch; `assert!(!map.unite_self())`.
    pub fn unite_self(&mut self) -> bool {
        false
    }
    /// Preserves the unchanged self-intersection branch; `assert!(!map.intersect_self())`.
    pub fn intersect_self(&mut self) -> bool {
        false
    }
    /// Clears self-subtraction, reporting whether any values existed; `map.subtract_self()`.
    pub fn subtract_self(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }
        self.clear();
        true
    }
    /// Clears self symmetric difference, reporting a cardinality change; `map.differ_self()`.
    pub fn differ_self(&mut self) -> bool {
        self.subtract_self()
    }
}
