//! Native removal keeps count until destruction completes, and Clear drains first-chain order.
use super::{
    BaseAllocator, BaseDoubleMapStorage, BaseMap, DoubleMapNode, DoubleStorage, MapLinkNode,
};
use crate::ncollection::{native_noexcept::terminate_on_unwind, node_release::release_typed_node};
impl<A, B> DoubleStorage<A, B> {
    pub(in crate::ncollection::double_map) fn destroy_unlinked(&mut self, node: *mut MapLinkNode) {
        self.release_node(node);
        self.topology.edit(|base| {
            base.decrement();
        });
    }
    pub(in crate::ncollection::double_map) fn clear(&mut self, release: bool) {
        if self.len() != 0 {
            for bucket in 0..=self.bucket_count() {
                self.destroy_chain(self.head(bucket, false));
                self.topology
                    .edit(|base| base.bucket_slots_mut()[bucket] = std::ptr::null_mut());
            }
        }
        self.topology.edit(|base| base.finish_destroy(release));
    }
    fn destroy_chain(&self, mut node: *mut MapLinkNode) {
        while !node.is_null() {
            let next = unsafe { (*node).next() };
            self.release_node(node);
            node = next;
        }
    }
    fn release_node(&self, node: *mut MapLinkNode) {
        // Native node destructors are noexcept; the bucket links above still name this node.
        terminate_on_unwind(|| unsafe {
            release_typed_node(node.cast::<DoubleMapNode<A, B>>(), self.allocator())
        });
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
