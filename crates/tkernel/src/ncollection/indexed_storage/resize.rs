//! Rehash follows native callback/link order; Rust unwind restores original links without hashing.
use super::{IndexedNode, IndexedStorage, MapLinkNode};
use crate::ncollection::{HashPolicy, next_map_prime};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
impl<N: IndexedNode> IndexedStorage<N> {
    pub(in crate::ncollection) fn resize(
        &mut self,
        requested: usize,
        hasher: &impl HashPolicy<N::Key>,
    ) {
        let Some(mut plan) = self.prepare_resize(requested) else {
            return;
        };
        let original = self.original_links();
        let result = catch_unwind(AssertUnwindSafe(|| {
            Self::relink(&original, &mut plan, hasher)
        }));
        if let Err(panic) = result {
            Self::restore_links(&original);
            resume_unwind(panic);
        }
        self.topology.edit(|base| base.base_mut().end_resize(plan));
    }
    fn prepare_resize(&mut self, requested: usize) -> Option<crate::ncollection::BaseMapResize> {
        let count = next_map_prime(requested).max(self.bucket_count());
        if count == self.bucket_count() && !self.topology.read().base().buckets().is_null() {
            return None;
        }
        self.topology
            .edit(|base| base.resize_index_table(count))
            .unwrap_or_else(|e| panic!("{e}"));
        let plan = self
            .topology
            .read()
            .base()
            .begin_resize(requested)
            .unwrap_or_else(|e| panic!("{e}"));
        Some(plan)
    }
    fn restore_links(links: &[(*mut MapLinkNode, *mut MapLinkNode)]) {
        for &(node, next) in links {
            unsafe {
                *(*node).next_mut() = next;
            }
        }
    }
    fn original_links(&self) -> Vec<(*mut MapLinkNode, *mut MapLinkNode)> {
        let mut links = Vec::with_capacity(self.len());
        for &head in self.topology.read().base().bucket_slots() {
            let mut node = head;
            while !node.is_null() {
                let next = unsafe { (*node).next() };
                links.push((node, next));
                node = next;
            }
        }
        links
    }
    fn relink(
        links: &[(*mut MapLinkNode, *mut MapLinkNode)],
        plan: &mut crate::ncollection::BaseMapResize,
        hasher: &impl HashPolicy<N::Key>,
    ) {
        for &(node, _) in links {
            let key = unsafe { &*N::key_ptr(node.cast()) };
            let bucket = hasher.hash(key) % plan.bucket_count() + 1;
            let head = &mut plan.bucket_slots_mut()[bucket];
            unsafe {
                *(*node).next_mut() = *head;
            }
            *head = node;
        }
    }
}
