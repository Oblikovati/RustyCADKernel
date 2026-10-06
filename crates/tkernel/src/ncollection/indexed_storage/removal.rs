//! Native removal publishes each metadata phase before calling user destruction.
use super::{BaseAllocator, BaseMap, IndexedNode, IndexedStorage, MapLinkNode};
use crate::ncollection::{
    BaseIndexedMapStorage, destructor_unwind::DeferredDestructorPanic,
    node_release::release_typed_node,
};
impl<N: IndexedNode> IndexedStorage<N> {
    pub(in crate::ncollection) fn unlink(&mut self, bucket: usize, target: *mut MapLinkNode) {
        let Some(previous) = self.chain_predecessor(bucket, target) else {
            return self.unlink_misrouted(target);
        };
        self.unlink_known(bucket, previous, target);
    }
    fn chain_predecessor(
        &self,
        bucket: usize,
        target: *mut MapLinkNode,
    ) -> Option<*mut MapLinkNode> {
        let (mut previous, mut node) = (std::ptr::null_mut(), self.head(bucket));
        while !node.is_null() {
            if node == target {
                return Some(previous);
            }
            previous = node;
            node = unsafe { (*node).next() };
        }
        None
    }
    fn unlink_misrouted(&mut self, target: *mut MapLinkNode) {
        // Safe interior-mutability or a stateful HashPolicy can break the key/hash contract.
        // Native behavior is unspecified there; Rust must still preserve graph memory safety.
        for bucket in 0..=self.bucket_count() {
            if let Some(previous) = self.chain_predecessor(bucket, target) {
                return self.unlink_known(bucket, previous, target);
            }
        }
        panic!(
            "indexed node {:p}; expected a reachable owned bucket node",
            target
        );
    }
    fn unlink_known(
        &mut self,
        bucket: usize,
        previous: *mut MapLinkNode,
        target: *mut MapLinkNode,
    ) {
        let next = unsafe { (*target).next() };
        self.topology.edit(|base| {
            if previous.is_null() {
                base.base_mut().bucket_slots_mut()[bucket] = next;
            } else {
                unsafe { *(*previous).next_mut() = next };
            }
        });
    }
    pub(in crate::ncollection) fn prepend(&mut self, bucket: usize, node: *mut MapLinkNode) {
        self.topology.edit(|base| {
            unsafe {
                *(*node).next_mut() = base.base().bucket_slots()[bucket];
            }
            base.base_mut().bucket_slots_mut()[bucket] = node;
        });
    }
    pub(in crate::ncollection) fn set_last_slot(&mut self, node: *mut MapLinkNode) {
        let slot = self.len() - 1;
        self.topology
            .edit(|base| base.index_slots_mut()[slot] = node);
    }
    pub(in crate::ncollection) fn destroy_last(&mut self, bucket: usize, node: *mut MapLinkNode) {
        self.unlink(bucket, node);
        let mut panics = DeferredDestructorPanic::default();
        panics.capture(|| unsafe { release_typed_node(node.cast::<N>(), self.allocator()) });
        self.topology.edit(|base| {
            base.base_mut().decrement();
        });
        panics.finish(Ok(()));
    }
    pub(in crate::ncollection) fn clear(&mut self, release: bool) {
        let mut panics = DeferredDestructorPanic::default();
        if self.len() != 0 {
            for bucket in 0..=self.bucket_count() {
                self.destroy_chain(self.head(bucket), &mut panics);
                self.topology
                    .edit(|base| base.base_mut().bucket_slots_mut()[bucket] = std::ptr::null_mut());
            }
        }
        self.topology.edit(|base| {
            base.base_mut().finish_destroy(release);
            base.clear_index_table(release);
        });
        panics.finish(Ok(()));
    }
    fn destroy_chain(&self, mut node: *mut MapLinkNode, panics: &mut DeferredDestructorPanic) {
        while !node.is_null() {
            let next = unsafe { (*node).next() };
            panics.capture(|| unsafe { release_typed_node(node.cast::<N>(), self.allocator()) });
            node = next;
        }
    }
    pub(in crate::ncollection) fn clear_with_allocator(
        &mut self,
        requested: Option<BaseAllocator>,
    ) {
        let release = requested.as_ref() != Some(self.allocator());
        self.clear(release);
        if release {
            self.topology
                .edit(|base| *base = BaseIndexedMapStorage::new(base.nb_buckets(), requested));
        }
    }
}
