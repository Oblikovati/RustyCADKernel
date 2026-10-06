//! Head publication and removal update the one native table/count, without an ownership registry.
use super::{BaseMap, DataMapNode, DataMapStorage, MapLinkNode};
use crate::ncollection::destructor_unwind::DeferredDestructorPanic;
impl<K, V> DataMapStorage<K, V> {
    pub(in super::super) fn insert(
        &mut self,
        bucket: usize,
        key: impl FnOnce() -> K,
        value: impl FnOnce() -> V,
    ) -> *mut MapLinkNode {
        let next = self.base.bucket_slots()[bucket];
        let owner = DataMapNode::allocate_with(self.base.allocator(), next, key, value);
        let (node, _allocator) = DataMapNode::into_raw(owner);
        self.base.bucket_slots_mut()[bucket] = node;
        self.base.increment();
        node
    }
    pub(in super::super) fn remove(&mut self, bucket: usize, target: *mut MapLinkNode) {
        let mut previous = std::ptr::null_mut();
        let mut node = self.base.bucket_slots()[bucket];
        while node != target {
            previous = node;
            node = unsafe { (*node).next() };
        }
        self.base.decrement();
        let next = unsafe { (*target).next() };
        self.unlink(bucket, previous, next);
        let mut allocator = self.base.allocator().clone();
        unsafe { DataMapNode::<K, V>::del_node(target, &mut allocator) };
    }
    fn unlink(&mut self, bucket: usize, previous: *mut MapLinkNode, next: *mut MapLinkNode) {
        if previous.is_null() {
            self.base.bucket_slots_mut()[bucket] = next;
        } else {
            // Touch only the link prefix, preserving existing item pointer provenance.
            unsafe { *(*previous).next_mut() = next };
        }
    }
    pub(in super::super) fn clear(&mut self, release: bool) {
        let mut panics = DeferredDestructorPanic::default();
        unsafe {
            self.base.destroy(
                |node, allocator| {
                    panics.capture(|| DataMapNode::<K, V>::del_node(node, allocator));
                },
                release,
            )
        };
        panics.finish(Ok(()));
    }
}
