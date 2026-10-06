//! Rehash visits native bucket chains; panic recovery restores links without repeat callbacks.
use super::*;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
impl<K, V, H: HashPolicy<K>, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn resize(&mut self, requested: usize) {
        let mut plan = self
            .base
            .begin_resize(requested)
            .unwrap_or_else(|e| panic!("{e}"));
        if !plan.is_resize() {
            return;
        }
        let links = self.original_links();
        let result = catch_unwind(AssertUnwindSafe(|| self.relink(&links, &mut plan)));
        if let Err(panic) = result {
            Self::restore_bucket_links(&links);
            resume_unwind(panic);
        }
        self.bucket_count = plan.bucket_count();
        self.base.end_resize(plan);
    }
    fn restore_bucket_links(links: &[(*mut MapLinkNode, *mut MapLinkNode)]) {
        // SAFETY: every snapshot node remains owned; restore only its bucket pointer.
        for &(node, next) in links {
            unsafe {
                *(*node).next_mut() = next;
            }
        }
    }
    fn original_links(&self) -> Vec<(*mut MapLinkNode, *mut MapLinkNode)> {
        let mut links = Vec::with_capacity(self.count);
        for &head in self.base.bucket_slots() {
            let mut node = head;
            while !node.is_null() {
                // SAFETY: the canonical bucket graph contains live acyclic chains.
                let next = unsafe { (*node).next() };
                links.push((node, next));
                node = next;
            }
        }
        links
    }
    fn relink(
        &self,
        links: &[(*mut MapLinkNode, *mut MapLinkNode)],
        plan: &mut crate::ncollection::BaseMapResize,
    ) {
        for &(node, _) in links {
            // SAFETY: only shared key and disjoint bucket-link fields are accessed.
            let hash = self.hasher.hash(unsafe { &*N::key(node) }) % plan.bucket_count() + 1;
            let head = &mut plan.bucket_slots_mut()[hash];
            unsafe {
                *(*node).next_mut() = *head;
            }
            *head = node;
        }
    }
}
