//! The observer reads the actual heap-stable native base between short mutation phases.
use crate::ncollection::map_topology::{MapMetadataBase, MapTopology, MapTopologyObserver};
use crate::ncollection::{BaseIndexedMapStorage, BaseMap};
pub(super) type IndexedTopology = MapTopology<BaseIndexedMapStorage>;
/// An independent read of actual map metadata, including during key destruction.
/// Example: `let observer = map.observer(); assert_eq!(observer.snapshot().size, map.len());`.
#[derive(Clone)]
pub struct IndexedMapObserver(pub(super) MapTopologyObserver<BaseIndexedMapStorage>);
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
impl IndexedMapObserver {
    /// Copies canonical metadata without holding a lock across user code; `observer.snapshot().size`.
    pub fn snapshot(&self) -> IndexedMapObservation {
        self.0.snapshot()
    }
}
impl MapMetadataBase for BaseIndexedMapStorage {
    type Observation = IndexedMapObservation;
    fn snapshot_metadata(&self) -> IndexedMapObservation {
        let base = self;
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

    fn exchange_metadata(&mut self, other: &mut Self) {
        self.exchange(other);
    }
}
