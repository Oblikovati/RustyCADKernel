//! Native removal keeps count until destruction completes, and Clear drains first-chain order.
use super::{
    BaseAllocator, BaseDoubleMapStorage, BaseMap, DoubleMapNode, DoubleStorage, MapLinkNode,
};
use crate::ncollection::{
    destructor_unwind::DeferredDestructorPanic, node_release::release_typed_node,
};
impl<A, B> DoubleStorage<A, B> {
    pub(in crate::ncollection::double_map) fn destroy_unlinked(&mut self, node: *mut MapLinkNode) {
        let mut panics = DeferredDestructorPanic::default();
        panics.capture(|| unsafe {
            release_typed_node(node.cast::<DoubleMapNode<A, B>>(), self.allocator())
        });
        self.topology.edit(|base| {
            base.decrement();
        });
        panics.finish(Ok(()));
    }
    pub(in crate::ncollection::double_map) fn clear(&mut self, release: bool) {
        let mut panics = DeferredDestructorPanic::default();
        if self.len() != 0 {
            for bucket in 0..=self.bucket_count() {
                self.destroy_chain(self.head(bucket, false), &mut panics);
                self.topology
                    .edit(|base| base.bucket_slots_mut()[bucket] = std::ptr::null_mut());
            }
        }
        self.topology.edit(|base| base.finish_destroy(release));
        panics.finish(Ok(()));
    }
    fn destroy_chain(&self, mut node: *mut MapLinkNode, panics: &mut DeferredDestructorPanic) {
        while !node.is_null() {
            let next = unsafe { (*node).next() };
            panics.capture(|| unsafe {
                release_typed_node(node.cast::<DoubleMapNode<A, B>>(), self.allocator())
            });
            node = next;
        }
    }
    pub(in crate::ncollection::double_map) fn clear_with_allocator(
        &mut self,
        requested: Option<BaseAllocator>,
    ) {
        // Native DoubleMap always releases tables, even when replacing with the same allocator.
        self.clear(true);
        self.topology
            .edit(|base| *base = BaseDoubleMapStorage::new(base.nb_buckets(), requested));
    }
}
