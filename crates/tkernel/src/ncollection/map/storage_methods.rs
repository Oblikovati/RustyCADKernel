//! Map's actual MapNode specialization shares canonical bucket ownership with DataMap.
use super::{Debug, HashPolicy, Map, MapLinkNode, MapNode};
use crate::ncollection::BasicMapIteratorSource;
impl<K: Debug, H: HashPolicy<K>> Map<K, H> {
    /// Borrows native const Buckets containing actual MapNode pointers; `map.buckets().is_null()`.
    pub fn buckets(&self) -> *const *mut MapLinkNode {
        self.storage.base.buckets()
    }
    /// Returns native mutable Buckets; `map.buckets_mut()`.
    /// Unsafe writes must preserve live acyclic ownership, node types, hashes and counts.
    pub fn buckets_mut(&mut self) -> *mut *mut MapLinkNode {
        self.storage.base.buckets_mut()
    }
    pub(super) fn place_key(&mut self, key: K, replace: bool) -> (bool, &K) {
        self.prepare_emplacement();
        let (bucket, node) = self.storage.lookup_with_bucket(&key, &self.hasher);
        if let Some(node) = node {
            if replace {
                *unsafe { self.storage.node_mut(node).change_value() } = key;
            }
            return (false, unsafe { self.storage.node(node).key() });
        }
        self.insert_key(bucket, || key)
    }
    fn insert_key(&mut self, bucket: usize, construct: impl FnOnce() -> K) -> (bool, &K) {
        // OCCT placement new obtains node storage before copying the borrowed key.
        let node = self
            .storage
            .allocate_node(bucket, |next| MapNode::construct(next, construct));
        (true, unsafe { self.storage.node(node).key() })
    }
}
impl<K: Debug + Clone, H: HashPolicy<K>> Map<K, H> {
    pub(super) fn place_key_ref(&mut self, key: &K) -> (bool, &K) {
        self.prepare_emplacement();
        let (bucket, node) = self.storage.lookup_with_bucket(key, &self.hasher);
        if let Some(node) = node {
            return (false, unsafe { self.storage.node(node).key() });
        }
        self.insert_key(bucket, || key.clone())
    }
}
impl<K, H> BasicMapIteratorSource for Map<K, H> {
    type Node = MapNode<K>;
    fn basic_bucket_slots(&self) -> Option<&[*mut MapLinkNode]> {
        if self.storage.base.buckets().is_null() {
            return None;
        }
        Some(self.storage.base.bucket_slots())
    }
    fn basic_first_node(&self) -> *mut MapNode<K> {
        std::ptr::null_mut()
    }
}
