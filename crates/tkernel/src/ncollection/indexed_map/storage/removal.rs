//! Native removal publishes each metadata phase before calling user destruction.
use super::{BaseAllocator, BaseMap, IndexedMapNode, IndexedStorage, MapLinkNode};
use crate::ncollection::{
    BaseIndexedMapStorage, destructor_unwind::DeferredDestructorPanic,
    node_release::release_typed_node,
};
impl<K> IndexedStorage<K> {
    pub(in crate::ncollection::indexed_map) fn unlink(
        &mut self,
        bucket: usize,
        target: *mut MapLinkNode,
    ) {
        let (mut previous, mut node) = (std::ptr::null_mut::<MapLinkNode>(), self.head(bucket));
        while node != target {
            previous = node;
            node = unsafe { (*node).next() };
        }
        let next = unsafe { (*target).next() };
        self.topology.edit(|base| {
            if previous.is_null() {
                base.base_mut().bucket_slots_mut()[bucket] = next;
            } else {
                unsafe { *(*previous).next_mut() = next };
            }
        });
    }
    pub(in crate::ncollection::indexed_map) fn prepend(
        &mut self,
        bucket: usize,
        node: *mut MapLinkNode,
    ) {
        self.topology.edit(|base| {
            unsafe {
                *(*node).next_mut() = base.base().bucket_slots()[bucket];
            }
            base.base_mut().bucket_slots_mut()[bucket] = node;
        });
    }
    pub(in crate::ncollection::indexed_map) fn set_last_slot(&mut self, node: *mut MapLinkNode) {
        let slot = self.len() - 1;
        self.topology
            .edit(|base| base.index_slots_mut()[slot] = node);
    }
    pub(in crate::ncollection::indexed_map) fn destroy_last(
        &mut self,
        bucket: usize,
        node: *mut MapLinkNode,
    ) {
        self.unlink(bucket, node);
        let mut panics = DeferredDestructorPanic::default();
        panics.capture(|| unsafe {
            release_typed_node(node.cast::<IndexedMapNode<K>>(), self.allocator())
        });
        self.topology.edit(|base| {
            base.base_mut().decrement();
        });
        panics.finish(Ok(()));
    }
    pub(in crate::ncollection::indexed_map) fn clear(&mut self, release: bool) {
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
            panics.capture(|| unsafe {
                release_typed_node(node.cast::<IndexedMapNode<K>>(), self.allocator())
            });
            node = next;
        }
    }
    pub(in crate::ncollection::indexed_map) fn clear_with_allocator(
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
