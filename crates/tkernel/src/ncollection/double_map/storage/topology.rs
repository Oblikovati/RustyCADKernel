//! Stable native metadata is observable between short mutation phases, including destruction.
use crate::ncollection::map_topology::{MapMetadataBase, MapTopology, MapTopologyObserver};
use crate::ncollection::{BaseDoubleMapStorage, BaseMap};
pub(super) type DoubleTopology = MapTopology<BaseDoubleMapStorage>;
/// Independent canonical metadata observation; `assert_eq!(map.observer().snapshot().size,map.len())`.
#[derive(Clone)]
pub struct DoubleMapObserver(pub(super) MapTopologyObserver<BaseDoubleMapStorage>);
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
impl DoubleMapObserver {
    /// Copies actual metadata without holding a lock across user code; `observer.snapshot().size`.
    pub fn snapshot(&self) -> DoubleMapObservation {
        self.0.snapshot()
    }
}
impl MapMetadataBase for BaseDoubleMapStorage {
    type Observation = DoubleMapObservation;
    fn snapshot_metadata(&self) -> DoubleMapObservation {
        let base = self;
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

    fn exchange_metadata(&mut self, other: &mut Self) {
        self.exchange(other);
    }
}
