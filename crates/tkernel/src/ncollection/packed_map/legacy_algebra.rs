//! The deprecated member set operations, each forwarding to `packed_map_algo` as the
//! native members forward to NCollection_PackedMapAlgo.
//!
//! Rust borrowing makes the native self-aliased calls (`aMap.Unite(aMap)`) unrepresentable,
//! so their early-return branches have no counterpart here.
use super::{PackedInteger, PackedMap};
use crate::ncollection::packed_map_algo as algo;

impl<T: PackedInteger> PackedMap<T> {
    /// Replaces this map with `left ∪ right`; `map.union(&left, &right)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::union")]
    pub fn union(&mut self, left: &Self, right: &Self) {
        algo::union(self, left, right);
    }
    /// Adds every key of `other`, reporting a change; `map.unite(&other)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::unite")]
    pub fn unite(&mut self, other: &Self) -> bool {
        algo::unite(self, other)
    }
    /// Replaces this map with `left ∩ right`; `map.intersection(&left, &right)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::intersection")]
    pub fn intersection(&mut self, left: &Self, right: &Self) {
        algo::intersection(self, left, right);
    }
    /// Keeps only keys also in `other`, reporting a change; `map.intersect(&other)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::intersect")]
    pub fn intersect(&mut self, other: &Self) -> bool {
        algo::intersect(self, other)
    }
    /// Replaces this map with `left \ right`; `map.subtraction(&left, &right)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::subtraction")]
    pub fn subtraction(&mut self, left: &Self, right: &Self) {
        algo::subtraction(self, left, right);
    }
    /// Removes the keys of `other`, reporting a change; `map.subtract(&other)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::subtract")]
    pub fn subtract(&mut self, other: &Self) -> bool {
        algo::subtract(self, other)
    }
    /// Replaces this map with the symmetric difference; `map.difference(&left, &right)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::difference")]
    pub fn difference(&mut self, left: &Self, right: &Self) {
        algo::difference(self, left, right);
    }
    /// Toggles the keys of `other`, reporting a change; `map.differ(&other)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::differ")]
    pub fn differ(&mut self, other: &Self) -> bool {
        algo::differ(self, other)
    }
    /// Same key set; `map.is_equal(&other)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::is_equal")]
    pub fn is_equal(&self, other: &Self) -> bool {
        algo::is_equal(self, other)
    }
    /// Every key of this map is in `other`; `map.is_subset(&other)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::is_subset")]
    pub fn is_subset(&self, other: &Self) -> bool {
        algo::is_subset(self, other)
    }
    /// Shares at least one key with `other`; `map.has_intersection(&other)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::has_intersection")]
    pub fn has_intersection(&self, other: &Self) -> bool {
        algo::has_intersection(self, other)
    }
    /// `Contains(const NCollection_PackedMap&)`: every key of `other` is here; `map.contains_map(&other)`.
    #[deprecated(note = "removed after OCCT 7.9; use packed_map_algo::contains")]
    pub fn contains_map(&self, other: &Self) -> bool {
        algo::contains(self, other)
    }
}
