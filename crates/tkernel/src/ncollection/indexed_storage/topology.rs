//! The observer reads the actual heap-stable native base between short mutation phases.
use crate::ncollection::{BaseAllocator, BaseIndexedMapStorage, BaseMap};
use std::{
    cell::UnsafeCell,
    sync::{Arc, Mutex, PoisonError},
};
struct IndexedCanonicalState {
    base: UnsafeCell<BaseIndexedMapStorage>,
    gate: Mutex<()>,
}
// SAFETY: all writes hold gate; observations only copy metadata and opaque addresses.
unsafe impl Send for IndexedCanonicalState {}
unsafe impl Sync for IndexedCanonicalState {}
impl std::panic::RefUnwindSafe for IndexedCanonicalState {}
pub(super) struct IndexedTopology(Arc<IndexedCanonicalState>);
/// An independent read of actual map metadata, including during key destruction.
/// Example: `let observer = map.observer(); assert_eq!(observer.snapshot().size, map.len());`.
#[derive(Clone)]
pub struct IndexedMapObserver(Arc<IndexedCanonicalState>);
/// Native metadata copied under a short lock; node identities cannot be dereferenced.
/// Example: `assert_eq!(map.observer().snapshot().size, 0);`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedMapObservation {
    /// Current native extent, including callback phases.
    pub size: usize,
    /// Native hash modulus.
    pub bucket_count: usize,
    /// Retained allocator identity.
    pub allocator: usize,
    /// Hash allocation identity, zero when absent.
    pub bucket_table: usize,
    /// Index allocation identity, zero when absent.
    pub index_table: usize,
    /// Actual bucket head identities, including reserved slot zero.
    pub buckets: Vec<usize>,
    /// Actual allocated index slot identities, including unused slots.
    pub indices: Vec<usize>,
}
impl IndexedTopology {
    pub(super) fn new(count: usize, allocator: BaseAllocator) -> Self {
        Self(Arc::new(IndexedCanonicalState {
            base: UnsafeCell::new(BaseIndexedMapStorage::new(count, Some(allocator))),
            gate: Mutex::new(()),
        }))
    }
    pub(super) fn read(&self) -> &BaseIndexedMapStorage {
        // SAFETY: structural mutation requires &mut owner; observers never mutate.
        unsafe { &*self.0.base.get() }
    }
    pub(super) fn edit<R>(&mut self, edit: impl FnOnce(&mut BaseIndexedMapStorage) -> R) -> R {
        let _lock = self.0.gate.lock().unwrap_or_else(PoisonError::into_inner);
        // SAFETY: owner exclusivity and gate exclude all competing metadata access.
        edit(unsafe { &mut *self.0.base.get() })
    }
    pub(super) fn exchange(&mut self, other: &mut Self) {
        // Stable observer identities remain attached to their original container.
        self.edit(|first| other.edit(|second| first.exchange(second)));
    }
    pub(super) fn observer(&self) -> IndexedMapObserver {
        IndexedMapObserver(self.0.clone())
    }
}
impl IndexedMapObserver {
    /// Copies canonical metadata without holding a lock across user code; `observer.snapshot().size`.
    pub fn snapshot(&self) -> IndexedMapObservation {
        let _lock = self.0.gate.lock().unwrap_or_else(PoisonError::into_inner);
        // SAFETY: every metadata mutation holds gate; identities alone escape.
        let base = unsafe { &*self.0.base.get() };
        IndexedMapObservation {
            size: base.size(),
            bucket_count: base.nb_buckets(),
            allocator: base.allocator().identity().addr(),
            bucket_table: base.base().buckets().addr(),
            index_table: base.index_buckets().addr(),
            buckets: base
                .base()
                .bucket_slots()
                .iter()
                .map(|p| p.addr())
                .collect(),
            indices: base.index_slots().iter().map(|p| p.addr()).collect(),
        }
    }
}
