//! Stable native metadata is observable between short mutation phases, including destruction.
use crate::ncollection::{BaseAllocator, BaseDoubleMapStorage, BaseMap};
use std::{
    cell::UnsafeCell,
    sync::{Arc, Mutex, PoisonError},
};
struct DoubleCanonicalState {
    base: UnsafeCell<BaseDoubleMapStorage>,
    gate: Mutex<()>,
}
// SAFETY: owner-only edits hold gate; observers copy metadata and opaque pointer identities.
// The base owns no payloads, and observer snapshots never dereference nodes, including freed heads.
unsafe impl Send for DoubleCanonicalState {}
unsafe impl Sync for DoubleCanonicalState {}
impl std::panic::RefUnwindSafe for DoubleCanonicalState {}
pub(super) struct DoubleTopology(Arc<DoubleCanonicalState>);
/// Independent canonical metadata observation; `assert_eq!(map.observer().snapshot().size,map.len())`.
#[derive(Clone)]
pub struct DoubleMapObserver(Arc<DoubleCanonicalState>);
/// Actual native metadata with opaque node identities; `map.observer().snapshot().first_buckets`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoubleMapObservation {
    /// Current native extent, including destructor callback phases.
    pub size: usize,
    /// Native shared hash modulus.
    pub bucket_count: usize,
    /// Allocator identity, retained until Clear callbacks finish.
    pub allocator: usize,
    /// First table allocation identity, zero when absent.
    pub first_table: usize,
    /// Second table identity inside the same allocation, zero when absent.
    pub second_table: usize,
    /// Primary bucket head identities, including reserved slot zero.
    pub first_buckets: Vec<usize>,
    /// Secondary bucket head identities, including reserved slot zero.
    pub second_buckets: Vec<usize>,
}
impl DoubleTopology {
    pub(super) fn new(count: usize, allocator: BaseAllocator) -> Self {
        Self(Arc::new(DoubleCanonicalState {
            base: UnsafeCell::new(BaseDoubleMapStorage::new(count, Some(allocator))),
            gate: Mutex::new(()),
        }))
    }
    pub(super) fn read(&self) -> &BaseDoubleMapStorage {
        // SAFETY: structural writes require exclusive owner access; observers only read.
        unsafe { &*self.0.base.get() }
    }
    pub(super) fn edit<R>(&mut self, edit: impl FnOnce(&mut BaseDoubleMapStorage) -> R) -> R {
        let _guard = self.0.gate.lock().unwrap_or_else(PoisonError::into_inner);
        // SAFETY: owner exclusivity and gate exclude competing accesses during metadata edits.
        edit(unsafe { &mut *self.0.base.get() })
    }
    pub(super) fn exchange(&mut self, other: &mut Self) {
        // Observers remain attached to their container's stable metadata identity.
        self.edit(|first| other.edit(|second| first.exchange(second)));
    }
    pub(super) fn observer(&self) -> DoubleMapObserver {
        DoubleMapObserver(self.0.clone())
    }
}
impl DoubleMapObserver {
    /// Copies actual metadata without holding a lock across user code; `observer.snapshot().size`.
    pub fn snapshot(&self) -> DoubleMapObservation {
        let _guard = self.0.gate.lock().unwrap_or_else(PoisonError::into_inner);
        // SAFETY: every metadata write holds gate; no node memory is read through these pointers.
        snapshot_base(unsafe { &*self.0.base.get() })
    }
}
fn snapshot_base(base: &BaseDoubleMapStorage) -> DoubleMapObservation {
    DoubleMapObservation {
        size: base.size(),
        bucket_count: base.nb_buckets(),
        allocator: base.allocator().identity().addr(),
        first_table: base.buckets().addr(),
        second_table: base.second_buckets().addr(),
        first_buckets: base.bucket_slots().iter().map(|p| p.addr()).collect(),
        second_buckets: base.second_slots().iter().map(|p| p.addr()).collect(),
    }
}
