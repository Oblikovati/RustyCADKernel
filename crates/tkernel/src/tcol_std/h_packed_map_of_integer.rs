//! `TColStd_HPackedMapOfInteger`: a `TColStd_PackedMapOfInteger` shared through handles.
use crate::{ncollection::PackedMap, standard::StandardTransient};
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A transient packed integer set whose edits every handle holder sees;
/// `Handle::new(HPackedMapOfInteger::new(1))`.
pub struct HPackedMapOfInteger {
    base: StandardTransient,
    // A lock gives the C++ `ChangeMap()` through a shared handle, which Rust lends only as `&`.
    map: RwLock<PackedMap<i32>>,
}
crate::impl_transient!(
    HPackedMapOfInteger,
    "TColStd_HPackedMapOfInteger",
    StandardTransient,
    base
);
impl Default for HPackedMapOfInteger {
    fn default() -> Self {
        Self::new(1)
    }
}
impl HPackedMapOfInteger {
    /// An empty map with `nb_buckets` initial buckets; a count below 1 means 1, as the
    /// `NCollection_PackedMap(int)` constructor; `HPackedMapOfInteger::new(10)`.
    pub fn new(nb_buckets: i32) -> Self {
        Self::from_map(PackedMap::new(nb_buckets.max(1) as usize))
    }
    /// Wraps a copy of `map`, as the copying constructor; `HPackedMapOfInteger::from_map_ref(&map)`.
    pub fn from_map_ref(map: &PackedMap<i32>) -> Self {
        Self::from_map(map.clone())
    }
    /// Takes ownership of `map`, as the moving constructor; `HPackedMapOfInteger::from_map(map)`.
    pub fn from_map(map: PackedMap<i32>) -> Self {
        Self {
            base: StandardTransient::default(),
            map: RwLock::new(map),
        }
    }
    /// Shared read access, as `Map()`; `object.map().contains(5)`.
    pub fn map(&self) -> RwLockReadGuard<'_, PackedMap<i32>> {
        self.map.read().unwrap_or_else(PoisonError::into_inner)
    }
    /// Exclusive edit access, as `ChangeMap()`; `object.change_map().add(5)`.
    pub fn change_map(&self) -> RwLockWriteGuard<'_, PackedMap<i32>> {
        self.map.write().unwrap_or_else(PoisonError::into_inner)
    }
}
