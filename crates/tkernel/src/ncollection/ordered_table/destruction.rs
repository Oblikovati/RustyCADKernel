//! Native bucket destruction order; node destructors are noexcept, as natively.
use super::*;
use crate::ncollection::{native_noexcept::terminate_on_unwind, node_release::release_typed_node};
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn clear(&mut self, release: bool) {
        let buckets = self.base.buckets_mut();
        let bucket_length = self.base.bucket_slots().len();
        for index in 0..bucket_length {
            // SAFETY: the bucket allocation remains live; node destruction cannot mutate it.
            unsafe {
                self.destroy_bucket(buckets.add(index));
            }
        }
        self.finish_clear(release);
    }
    unsafe fn destroy_bucket(&self, head: *mut *mut MapLinkNode) {
        // SAFETY: caller supplies a live unique bucket slot; cache links before node destruction.
        let mut node = unsafe { *head };
        while !node.is_null() {
            let next = unsafe { (*node).next() };
            terminate_on_unwind(|| unsafe {
                release_typed_node(node.cast::<N>(), self.resource())
            });
            node = next;
        }
        unsafe {
            *head = std::ptr::null_mut();
        }
    }
    pub(super) fn finish_clear(&mut self, release: bool) {
        self.base.finish_destroy(release);
        self.slots.clear();
        self.free.clear();
        self.identities.clear();
        self.first = None;
        self.last = None;
        self.count = 0;
    }
}
impl<K, V, H, N: StorageNode<K, V>> Drop for OrderedTable<K, V, H, N> {
    fn drop(&mut self) {
        if self.incomplete_copy {
            self.abandon_incomplete_copy();
            return;
        }
        self.clear(true);
    }
}
