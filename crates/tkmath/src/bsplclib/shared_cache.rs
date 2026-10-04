//! Shared ownership for mutable spline caches stored in OCCT-style handles.
use rustycad_tkernel::standard::{RuntimeType, Transient};
use std::sync::{Arc, Mutex, MutexGuard};

use super::Cache;

/// Generic nullable owner for a shared, mutable cache object.
///
/// Clones retain one cache object; `cache.with_mut(|value| update(value))`.
pub struct SharedCache<T>(Option<Arc<Mutex<T>>>);

impl<T> SharedCache<T> {
    /// Owns one value behind shared mutation; `SharedCache::new(cache_value)`.
    pub fn new(cache: T) -> Self {
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

    /// Reads the shared value while locked; `cache.with(|value| inspect(value))`.
    pub fn with<R>(&self, inspect: impl FnOnce(&T) -> R) -> Option<R> {
        self.0.as_ref().map(|cache| inspect(&lock_cache(cache)))
    }

    /// Mutates the shared value while locked; `cache.with_mut(|value| update(value))`.
    pub fn with_mut<R>(&self, mutate: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.0.as_ref().map(|cache| mutate(&mut lock_cache(cache)))
    }
}

impl<T> Clone for SharedCache<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Transient> SharedCache<T> {
    /// Reads the cached object's runtime descriptor; `cache.dynamic_type().unwrap().name()`.
    pub fn dynamic_type(&self) -> Option<&'static RuntimeType> {
        self.with(Transient::dynamic_type)
    }

    /// Checks exact cached-object type; `cache.is_instance(Cache::static_type())`.
    pub fn is_instance(&self, exact: &RuntimeType) -> bool {
        let Some(matches) = self.with(|value| value.is_instance(exact)) else {
            return false;
        };
        matches
    }

    /// Checks cached-object ancestry; `cache.is_kind(StandardTransient::static_type())`.
    pub fn is_kind(&self, base: &RuntimeType) -> bool {
        let Some(matches) = self.with(|value| value.is_kind(base)) else {
            return false;
        };
        matches
    }
}

impl<T> Default for SharedCache<T> {
    fn default() -> Self {
        Self(None)
    }
}

/// Nullable shared owner corresponding to `Handle<BSplCLib_Cache>`.
#[derive(Clone, Default)]
pub struct SharedBSplineCache(SharedCache<Cache>);

impl SharedBSplineCache {
    /// Owns one cache behind shared mutation; `SharedBSplineCache::new(cache_value)`.
    pub fn new(cache: Cache) -> Self {
        Self(SharedCache::new(cache))
    }

    /// Tests for the native null-handle state; `cache.is_null()`.
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    /// Drops this handle without affecting other copies; `cache.nullify()`.
    pub fn nullify(&mut self) {
        self.0.nullify();
    }

    /// Compares shared-object identity; `left.is_same(&right)`.
    pub fn is_same(&self, other: &Self) -> bool {
        self.0.is_same(&other.0)
    }

    /// Reports the underlying cache class; `cache.dynamic_type().unwrap().name()`.
    pub fn dynamic_type(&self) -> Option<&'static RuntimeType> {
        if self.is_null() {
            return None;
        }
        self.0.dynamic_type()
    }

    /// Checks exact underlying cache type; `cache.is_instance(Cache::static_type())`.
    pub fn is_instance(&self, exact: &RuntimeType) -> bool {
        if self.is_null() {
            return false;
        }
        self.0.is_instance(exact)
    }

    /// Checks underlying cache ancestry; `cache.is_kind(StandardTransient::static_type())`.
    pub fn is_kind(&self, base: &RuntimeType) -> bool {
        if self.is_null() {
            return false;
        }
        self.0.is_kind(base)
    }

    /// Reads the shared cache while locked; `cache.with(|value| value.params().degree)`.
    pub fn with<R>(&self, inspect: impl FnOnce(&Cache) -> R) -> Option<R> {
        self.0.with(inspect)
    }

    /// Mutates the shared cache while locked; `cache.with_mut(|value| value.d0(0.5))`.
    pub fn with_mut<R>(&self, mutate: impl FnOnce(&mut Cache) -> R) -> Option<R> {
        self.0.with_mut(mutate)
    }
}

fn lock_cache<T>(cache: &Mutex<T>) -> MutexGuard<'_, T> {
    cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
