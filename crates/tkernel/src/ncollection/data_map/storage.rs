//! DataMap construction specializes the shared canonical chain owner with its actual node type.
use crate::ncollection::{DataMapNode, MapLinkNode, owned_bucket_chains::OwnedBucketChains};
pub(super) type DataMapStorage<K, V> = OwnedBucketChains<DataMapNode<K, V>>;
impl<K, V> OwnedBucketChains<DataMapNode<K, V>> {
    pub(in crate::ncollection) fn insert(
        &mut self,
        bucket: usize,
        key: impl FnOnce() -> K,
        value: impl FnOnce() -> V,
    ) -> *mut MapLinkNode {
        // Native placement new allocates before constructing inherited value, then its key.
        self.allocate_node(bucket, |next| {
            DataMapNode::construct_parts(next, key, value)
        })
    }
}
