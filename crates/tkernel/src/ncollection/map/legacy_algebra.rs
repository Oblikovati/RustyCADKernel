//! Still-public native deprecated methods delegate to the owned MapAlgo implementation.
use super::{Debug, HashPolicy, Map};
use crate::ncollection::map_algo;

impl<K: Debug, H: HashPolicy<K>> Map<K, H> {
    /// Compares all keys, as native IsEqual; `a.is_equal(&b)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        map_algo::is_equal(self, other)
    }
    /// Checks every key of another map, as Contains(Map); `a.contains_map(&b)`.
    pub fn contains_map(&self, other: &Self) -> bool {
        map_algo::contains(self, other)
    }
    /// Tests whether any key occurs in both maps; `a.has_intersection(&b)`.
    pub fn has_intersection(&self, other: &Self) -> bool {
        map_algo::has_intersection(self, other)
    }
    /// Removes keys found in another map, returning size change; `a.subtract(&b)`.
    pub fn subtract(&mut self, other: &Self) -> bool {
        map_algo::subtract(self, other)
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> Map<K, H> {
    /// Adds another map's keys and reports size change; `a.unite(&b)`.
    pub fn unite(&mut self, other: &Self) -> bool {
        map_algo::unite(self, other)
    }
    /// Replaces keys with the symmetric difference of two maps; `out.difference(&a, &b)`.
    pub fn difference(&mut self, left: &Self, right: &Self) {
        map_algo::difference(self, left, right);
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> Map<K, H> {
    /// Replaces keys with the union of two maps; `out.union(&a, &b)`.
    pub fn union(&mut self, left: &Self, right: &Self) {
        map_algo::union(self, left, right);
    }
    /// Replaces keys with the common keys of two maps; `out.intersection(&a, &b)`.
    pub fn intersection(&mut self, left: &Self, right: &Self) {
        map_algo::intersection(self, left, right);
    }
    /// Replaces keys with left keys absent from right; `out.subtraction(&a, &b)`.
    pub fn subtraction(&mut self, left: &Self, right: &Self) {
        map_algo::subtraction(self, left, right);
    }
}
impl<K: Debug + Clone, H: HashPolicy<K> + Default> Map<K, H> {
    /// Retains common keys using the native fresh default hasher; `a.intersect(&b)`.
    pub fn intersect(&mut self, other: &Self) -> bool {
        map_algo::intersect(self, other)
    }
}
impl<K: Debug + Clone, H: HashPolicy<K> + Default> Map<K, H> {
    /// Rebuilds the symmetric difference and reports size change; `a.differ(&b)`.
    pub fn differ(&mut self, other: &Self) -> bool {
        map_algo::differ(self, other)
    }
}
