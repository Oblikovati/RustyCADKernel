//! Head publication and removal update the one native table/count, without an ownership registry.
use super::{BaseMap, CanonicalMapNode, MapLinkNode, OwnedBucketChains};
use crate::ncollection::{destructor_unwind::DeferredDestructorPanic, node_owner::NativeNodeOwner};
impl<N: CanonicalMapNode> OwnedBucketChains<N> {
    pub(in crate::ncollection) fn allocate_node(
        &mut self,
        bucket: usize,
        construct: impl FnOnce(*mut MapLinkNode) -> N,
    ) -> *mut MapLinkNode {
        let next = self.base.bucket_slots()[bucket];
        let owner = self
            .base
            .allocator()
            .allocate_value_with(|| construct(next));
        let (pointer, _allocator) = NativeNodeOwner::from_allocated(owner).into_parts();
        let node = pointer.cast();
        self.base.bucket_slots_mut()[bucket] = node;
        self.base.increment();
        node
    }
    pub(in crate::ncollection) fn remove(&mut self, bucket: usize, target: *mut MapLinkNode) {
        let mut previous = std::ptr::null_mut();
        let mut node = self.base.bucket_slots()[bucket];
        while node != target {
            previous = node;
            node = unsafe { (*node).next() };
        }
        self.base.decrement();
        let next = unsafe { (*target).next() };
        self.unlink(bucket, previous, next);
        unsafe { N::release_owned(target.cast::<N>(), self.base.allocator()) };
    }
    fn unlink(&mut self, bucket: usize, previous: *mut MapLinkNode, next: *mut MapLinkNode) {
        if previous.is_null() {
            self.base.bucket_slots_mut()[bucket] = next;
        } else {
            // Touch only the link prefix, preserving existing item pointer provenance.
            unsafe { *(*previous).next_mut() = next };
        }
    }
    pub(in crate::ncollection) fn clear(&mut self, release: bool) {
        let mut panics = DeferredDestructorPanic::default();
        unsafe {
            self.base.destroy(
                |node, allocator| {
                    panics.capture(|| N::release_owned(node.cast::<N>(), allocator));
                },
                release,
            )
        };
        panics.finish(Ok(()));
    }
}
