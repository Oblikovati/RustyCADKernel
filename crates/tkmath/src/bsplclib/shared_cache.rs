//! Shared ownership for mutable B-spline caches stored in OCCT-style handles.
use std::sync::{Arc, Mutex, MutexGuard};

use super::Cache;

/// Nullable shared payload corresponding to `Handle<BSplCLib_Cache>`.
///
/// Clones retain one cache object; `cache.with_mut(|value| value.d0(0.5))`.
#[derive(Clone, Default)]
pub struct SharedBSplineCache(Option<Arc<Mutex<Cache>>>);

impl SharedBSplineCache {
    /// Owns one cache behind shared mutation; `SharedBSplineCache::new(cache_value)`.
    pub fn new(cache: Cache) -> Self {
        Self(Some(Arc::new(Mutex::new(cache))))
    }

    /// Tests for the native null-handle state; `cache.is_null()`.
    pub fn is_null(&self) -> bool {
        self.0.is_none()
    }

    /// Drops this handle without affecting other copies; `cache.nullify()`.
    pub fn nullify(&mut self) {
        self.0 = None;
    }

    /// Compares shared-object identity; `left.is_same(&right)`.
    pub fn is_same(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            (None, None) => true,
            _ => false,
        }
    }

    /// Reads the shared cache while locked; `cache.with(|value| value.params().degree)`.
    pub fn with<R>(&self, inspect: impl FnOnce(&Cache) -> R) -> Option<R> {
        self.0.as_ref().map(|cache| inspect(&lock_cache(cache)))
    }

    /// Mutates the shared cache while locked; `cache.with_mut(|value| value.d0(0.5))`.
    pub fn with_mut<R>(&self, mutate: impl FnOnce(&mut Cache) -> R) -> Option<R> {
        self.0.as_ref().map(|cache| mutate(&mut lock_cache(cache)))
    }
}

fn lock_cache(cache: &Mutex<Cache>) -> MutexGuard<'_, Cache> {
    cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
