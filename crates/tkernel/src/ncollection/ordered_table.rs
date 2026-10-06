//! Native ordered nodes and bucket/order links are the sole owning graph.
#![allow(unsafe_code)]
use super::{BaseAllocator, BaseMap, BaseMapStorage, HashPolicy, MapLinkNode, OrderedDataMapNode};
use std::{collections::HashMap, marker::PhantomData};
mod node_contract;
use node_contract::StorageNode;
pub(super) struct OrderedTable<K, V, H, N: StorageNode<K, V> = OrderedDataMapNode<K, V>> {
    // These non-owning identity slots preserve safe detached cursor validation and reuse.
    pub slots: Vec<Option<*mut MapLinkNode>>,
    free: Vec<usize>,
    identities: HashMap<usize, usize>,
    // Derived hasher destruction precedes base cleanup on constructor unwind (OCCT 3d097a0).
    pub hasher: H,
    base: BaseMapStorage,
    incomplete_copy: bool,
    pub count: usize,
    pub bucket_count: usize,
    pub first: Option<usize>,
    pub last: Option<usize>,
    ownership: PhantomData<Box<(K, V, N)>>,
}
impl<K, V, H: HashPolicy<K>, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn new(hasher: H, buckets: usize, resource: BaseAllocator) -> Self {
        Self::new_with_policy(|| hasher, buckets, resource)
    }
    pub fn new_with_policy(
        make_policy: impl FnOnce() -> H,
        buckets: usize,
        resource: BaseAllocator,
    ) -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            identities: HashMap::new(),
            base: BaseMapStorage::new(buckets, Some(resource)),
            incomplete_copy: false,
            count: 0,
            bucket_count: buckets,
            first: None,
            last: None,
            hasher: make_policy(),
            ownership: PhantomData,
        }
    }
    pub fn lookup(&self, key: &K) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        let hash = self.hasher.hash(key) % self.bucket_count + 1;
        self.lookup_at_hash(key, hash)
    }
    fn lookup_at_hash(&self, key: &K, hash: usize) -> Option<usize> {
        let mut node = self.base.bucket_slots()[hash];
        while !node.is_null() {
            // SAFETY: bucket links contain only owned live nodes; shared projection excludes values.
            if self.hasher.equal(unsafe { &*N::key(node) }, key) {
                return self.slot_of(node);
            }
            node = unsafe { (*node).next() };
        }
        None
    }
    pub fn remove(&mut self, key: &K) -> bool {
        if self.count == 0 {
            return false;
        }
        let hash = self.hasher.hash(key) % self.bucket_count + 1;
        let Some(slot) = self.lookup_at_hash(key, hash) else {
            return false;
        };
        self.remove_slot(slot, hash);
        true
    }
    pub fn clear_with_allocator(&mut self, resource: Option<BaseAllocator>) {
        let release = resource.as_ref() != Some(self.resource());
        self.clear(release);
        if release {
            self.base = BaseMapStorage::new(self.bucket_count, resource);
        }
    }
}
impl<K: Clone, V: Clone, H: HashPolicy<K>, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn assign(&mut self, source: &Self, replace_values: bool) {
        self.clear(false);
        if source.count == 0 {
            return;
        }
        self.resize(source.count - 1);
        for (key, value) in source.items() {
            self.place_borrowed(key, value, replace_values);
        }
    }
}
impl<K: Clone, V: Clone, H: HashPolicy<K> + Clone, N: StorageNode<K, V>> Clone
    for OrderedTable<K, V, H, N>
{
    fn clone(&self) -> Self {
        self.clone_with(false)
    }
}
/// Signed native bucket count validation; `buckets_from_int(-1)` names the invalid value.
pub(super) fn buckets_from_int(buckets: i32) -> Result<usize, crate::standard::Failure> {
    usize::try_from(buckets).map_err(|_| {
        crate::standard::Failure::with_type(
            "Standard_OutOfRange",
            &format!("bucket count {buckets}; expected a non-negative count"),
            "",
        )
    })
}
mod iteration;
pub(super) use iteration::OrderedItems;
mod nodes;
pub use nodes::{OrderedNodeMut, OrderedNodeRef};
mod borrowed;
mod mutable;
pub(super) use mutable::OrderedItemsMut;
mod destruction;
mod graph;
mod placement;
mod projection;
mod resize;
mod traits;

#[cfg(test)]
mod canonical_contracts;

mod base_bridge;
pub(super) use base_bridge::ordered_base_bridge;

mod policy_construction;
pub(super) use policy_construction::ordered_policy_construction;

mod policy_exchange;
pub(super) use policy_exchange::ordered_policy_exchange;

mod copy_construction;
pub(super) use copy_construction::ordered_default_construction;
