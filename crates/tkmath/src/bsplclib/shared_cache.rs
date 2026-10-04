//! Shared ownership for mutable spline caches stored in OCCT-style handles.
use rustycad_tkernel::standard::{Handle, RuntimeType, RuntimeTyped, Transient};

use super::Cache;

/// Generic nullable owner for a shared, mutable cache object.
///
/// Clones retain one Handle-owned cache object; `cache.with_mut(|value| value.build_cache(...))`.
pub struct SharedCache<T: Transient + RuntimeTyped>(Handle<T>);

impl<T: Transient + RuntimeTyped> SharedCache<T> {
    /// Owns one value behind shared mutation; `SharedCache::new(cache_value)`.
    pub fn new(cache: T) -> Self {
        Self(Handle::new(cache))
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
        self.0.object_address() == other.0.object_address()
    }

    /// Reads the shared object; `cache.with(|value| inspect(value))`.
    pub fn with<R>(&self, inspect: impl FnOnce(&T) -> R) -> Option<R> {
        self.0.get().map(inspect)
    }

    /// Calls an interior-mutating cache operation; `cache.with_mut(|value| value.build_cache(...))`.
    pub fn with_mut<R>(&self, mutate: impl FnOnce(&T) -> R) -> Option<R> {
        self.0.get().map(mutate)
    }

    /// Returns an owning base-projectable handle; `let owner = cache.handle()`.
    pub fn handle(&self) -> Handle<T> {
        self.0.clone()
    }

    /// Reads stable object identity without making a temporary owning copy; `cache.object_address()`.
    pub fn object_address(&self) -> *const () {
        self.0.object_address()
    }

    /// Reads the embedded Standard_Transient count; `cache.ref_count()`.
    pub fn ref_count(&self) -> i32 {
        self.0.ref_count()
    }
}

impl<T: Transient + RuntimeTyped> Clone for SharedCache<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Transient + RuntimeTyped> SharedCache<T> {
    /// Reads the cached object's runtime descriptor; `cache.dynamic_type().unwrap().name()`.
    pub fn dynamic_type(&self) -> Option<&'static RuntimeType> {
        self.0.dynamic_type()
    }

    /// Checks exact cached-object type; `cache.is_instance(Cache::static_type())`.
    pub fn is_instance(&self, exact: &RuntimeType) -> bool {
        self.0.is_instance(exact)
    }

    /// Checks cached-object ancestry; `cache.is_kind(StandardTransient::static_type())`.
    pub fn is_kind(&self, base: &RuntimeType) -> bool {
        self.0.is_kind(base)
    }
}

impl<T: Transient + RuntimeTyped> Default for SharedCache<T> {
    fn default() -> Self {
        Self(Handle::default())
    }
}

#[cfg(test)]
mod tests {
    use super::SharedBSplineCache;
    use crate::bsplclib::Cache;
    use crate::gp::Point3;
    use rustycad_tkernel::ncollection::Array1;
    use rustycad_tkernel::standard::{Handle, StandardTransient};
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    fn curve_cache() -> SharedBSplineCache {
        let mut knots = Array1::with_bounds(1, 4).expect("small knot vector");
        knots.as_mut_slice().copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
        let mut poles = Array1::with_bounds(1, 2).expect("small pole vector");
        poles[1] = Point3::new(0.0, 0.0, 0.0);
        poles[2] = Point3::new(1.0, 0.0, 0.0);
        SharedBSplineCache::new(Cache::new(1, false, &knots, &poles, None).expect("cache"))
    }

    #[test]
    fn owning_casts_update_embedded_count_without_taking_payload_lock() {
        let owner = Arc::new(curve_cache());
        assert_eq!(owner.ref_count(), 1);
        let (sender, receiver) = mpsc::channel();
        let worker = Arc::clone(&owner);
        let thread = std::thread::spawn(move || clone_while_locked(worker, sender));
        let (clone, base) = receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("lock-independent handle clone");
        assert_eq!(clone.ref_count(), 3);
        assert_eq!(base.ref_count(), 3);
        assert!(owner.is_same(&clone));
        thread.join().expect("clone worker completes");
        drop((clone, base));
        assert_eq!(owner.ref_count(), 1);
    }

    fn clone_while_locked(
        owner: Arc<SharedBSplineCache>,
        sender: mpsc::Sender<(SharedBSplineCache, Handle<StandardTransient>)>,
    ) {
        owner.with(|cache| {
            let _payload = cache.lock_state();
            let clone = owner.as_ref().clone();
            let base = clone.handle().cast::<StandardTransient>();
            sender
                .send((clone, base))
                .expect("receiver remains available");
        });
    }

    #[test]
    fn null_handles_and_clone_identity_follow_handle_semantics() {
        let mut source = curve_cache();
        let copy = source.clone();
        assert_eq!(source.ref_count(), 2);
        assert!(source.is_same(&copy));
        source.nullify();
        assert!(source.is_null());
        assert_eq!(source.ref_count(), 0);
        assert_eq!(copy.ref_count(), 1);
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

    /// Returns an owning base-projectable handle; `let transient = cache.handle().cast::<StandardTransient>()`.
    pub fn handle(&self) -> Handle<Cache> {
        self.0.handle()
    }

    /// Reads stable object identity without making a temporary owning copy; `cache.object_address()`.
    pub fn object_address(&self) -> *const () {
        self.0.object_address()
    }

    /// Reads the embedded Standard_Transient count; `cache.ref_count()`.
    pub fn ref_count(&self) -> i32 {
        self.0.ref_count()
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

    /// Reads the shared cache object; `cache.with(|value| value.params().degree)`.
    pub fn with<R>(&self, inspect: impl FnOnce(&Cache) -> R) -> Option<R> {
        self.0.with(inspect)
    }

    /// Calls an interior-mutating cache operation; `cache.with_mut(|value| value.build_cache(...))`.
    pub fn with_mut<R>(&self, mutate: impl FnOnce(&Cache) -> R) -> Option<R> {
        self.0.with_mut(mutate)
    }
}
