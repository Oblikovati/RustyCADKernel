//! Hash callbacks complete before canonical links change, so a Rust panic cannot corrupt ownership.
use super::{CanonicalMapNode, MapLinkNode, OwnedBucketChains};
use crate::ncollection::{BaseMapResize, HashPolicy};
impl<N: CanonicalMapNode> OwnedBucketChains<N> {
    pub(in crate::ncollection) fn resize(&mut self, size: usize, hasher: &impl HashPolicy<N::Key>) {
        let mut plan = self
            .base
            .begin_resize(size)
            .unwrap_or_else(|failure| panic!("{failure}"));
        if !plan.is_resize() {
            return;
        }
        let destinations = self.rehash_destinations(plan.bucket_count(), hasher);
        Self::relink_destinations(&mut plan, destinations);
        self.base.end_resize(plan);
    }
    fn rehash_destinations(
        &self,
        count: usize,
        hasher: &impl HashPolicy<N::Key>,
    ) -> Vec<(*mut MapLinkNode, usize)> {
        let mut destinations = Vec::new();
        for &head in self.base.bucket_slots() {
            let mut node = head;
            while !node.is_null() {
                let key = unsafe { self.node(node).map_key() };
                destinations.push((node, hasher.hash(key) % count + 1));
                node = unsafe { (*node).next() };
            }
        }
        destinations
    }
    fn relink_destinations(plan: &mut BaseMapResize, destinations: Vec<(*mut MapLinkNode, usize)>) {
        for (node, bucket) in destinations {
            let head = &mut plan.bucket_slots_mut()[bucket];
            // Native rehash prepends in old bucket/chain order; only the header is reborrowed.
            unsafe { *(*node).next_mut() = *head };
            *head = node;
        }
    }
}
