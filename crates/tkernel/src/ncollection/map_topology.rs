//! Shared synchronization for the two audited native metadata bases, never payload ownership.
#![allow(unsafe_code)]
use super::{BaseDoubleMapStorage, BaseIndexedMapStorage};
use std::{
    cell::UnsafeCell,
    sync::{Arc, Mutex, PoisonError},
};
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::BaseDoubleMapStorage {}
    impl Sealed for super::BaseIndexedMapStorage {}
}
/// The whitelist excludes user implementations: observations contain owned numeric metadata only.
pub(super) trait MapMetadataBase: sealed::Sealed {
    type Observation: Send + Sync + 'static;
    fn snapshot_metadata(&self) -> Self::Observation;
    fn exchange_metadata(&mut self, other: &mut Self);
}
struct MapTopologyState<S: MapMetadataBase> {
    base: UnsafeCell<S>,
    gate: Mutex<()>,
}
// SAFETY: the sealed bases own metadata allocations, not payloads. Unique-owner edits hold gate;
// observers copy only numeric identities, including transient freed heads during native Clear.
// The typed owner drains payloads before releasing its handle; a surviving observer never owns them.
unsafe impl<S: MapMetadataBase> Send for MapTopologyState<S> {}
unsafe impl<S: MapMetadataBase> Sync for MapTopologyState<S> {}
impl<S: MapMetadataBase> std::panic::RefUnwindSafe for MapTopologyState<S> {}
/// The sole structural owner. Deliberately not Clone and never obtainable from an observer.
pub(super) struct MapTopology<S: MapMetadataBase>(Arc<MapTopologyState<S>>);
pub(super) struct MapTopologyObserver<S: MapMetadataBase>(Arc<MapTopologyState<S>>);
impl<S: MapMetadataBase> MapTopology<S> {
    pub(super) fn new(base: S) -> Self {
        Self(Arc::new(MapTopologyState {
            base: UnsafeCell::new(base),
            gate: Mutex::new(()),
        }))
    }
    pub(super) fn read(&self) -> &S {
        // SAFETY: structural mutation requires exclusive owner access; observers only read.
        unsafe { &*self.0.base.get() }
    }
    pub(super) fn edit<R>(&mut self, operation: impl FnOnce(&mut S) -> R) -> R {
        let _guard = self.0.gate.lock().unwrap_or_else(PoisonError::into_inner);
        // SAFETY: owner exclusivity and gate exclude competing accesses during metadata edits.
        // Internal closures must not invoke user code or let a borrowed state escape.
        operation(unsafe { &mut *self.0.base.get() })
    }
    pub(super) fn exchange(&mut self, other: &mut Self) {
        // Distinct unique owners cannot share an Arc, so these two locks cannot self-deadlock.
        // Observers remain attached to their container's stable metadata identity.
        self.edit(|first| other.edit(|second| first.exchange_metadata(second)));
    }
    pub(super) fn observer(&self) -> MapTopologyObserver<S> {
        MapTopologyObserver(self.0.clone())
    }
}
impl<S: MapMetadataBase> Clone for MapTopologyObserver<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<S: MapMetadataBase> MapTopologyObserver<S> {
    pub(super) fn snapshot(&self) -> S::Observation {
        let _guard = self.0.gate.lock().unwrap_or_else(PoisonError::into_inner);
        // SAFETY: every write holds gate; sealed builders never dereference payload pointers.
        // No arbitrary caller closure can run under the lock or leak a metadata reference.
        unsafe { &*self.0.base.get() }.snapshot_metadata()
    }
}
