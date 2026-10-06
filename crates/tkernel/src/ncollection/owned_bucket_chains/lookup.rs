//! Empty native maps skip hashing; insertion callers prepare real nonzero buckets first.
use super::{BaseMap, CanonicalMapNode, MapLinkNode, OwnedBucketChains};
use crate::ncollection::HashPolicy;
impl<N: CanonicalMapNode> OwnedBucketChains<N> {
    pub(in crate::ncollection) fn lookup(
        &self,
        key: &N::Key,
        hasher: &impl HashPolicy<N::Key>,
    ) -> Option<(usize, *mut MapLinkNode)> {
        // Native lookup skips user hashing when Clear(false) retained an empty bucket table.
        if self.base.size() == 0 {
            return None;
        }
        let (bucket, node) = self.lookup_with_bucket(key, hasher);
        node.map(|node| (bucket, node))
    }
    pub(in crate::ncollection) fn lookup_with_bucket(
        &self,
        key: &N::Key,
        hasher: &impl HashPolicy<N::Key>,
    ) -> (usize, Option<*mut MapLinkNode>) {
        let bucket = hasher.hash(key) % self.base.nb_buckets() + 1;
        let mut node = self.base.bucket_slots()[bucket];
        while !node.is_null() {
            if hasher.equal(unsafe { self.node(node).map_key() }, key) {
                return (bucket, Some(node));
            }
            node = unsafe { (*node).next() };
        }
        (bucket, None)
    }
}
