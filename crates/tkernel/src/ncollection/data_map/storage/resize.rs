//! Hash callbacks complete before canonical links change, so a Rust panic cannot corrupt ownership.
use super::{DataMapStorage, MapLinkNode};
use crate::ncollection::{BaseMapResize, HashPolicy};
impl<K, V> DataMapStorage<K, V> {
    pub(in super::super) fn resize(&mut self, size: usize, hasher: &impl HashPolicy<K>) {
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
        hasher: &impl HashPolicy<K>,
    ) -> Vec<(*mut MapLinkNode, usize)> {
        let mut destinations = Vec::new();
        for &head in self.base.bucket_slots() {
            let mut node = head;
            while !node.is_null() {
                let key = unsafe { self.node(node).key() };
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
