//! Native bucket destruction order; Rust unwinding finishes every allocation before resuming.
use super::*;
use crate::ncollection::{
    destructor_unwind::DeferredDestructorPanic, node_release::release_typed_node,
};
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn clear(&mut self, release: bool) {
        let mut panics = DeferredDestructorPanic::default();
        for head in self.base.bucket_slots_mut() {
            let mut node = *head;
            while !node.is_null() {
                // SAFETY: cache the next live link before destroying only this node.
                let next = unsafe { (*node).next() };
                panics.capture(|| unsafe { release_typed_node(node.cast::<N>(), &self.resource) });
                node = next;
            }
            *head = std::ptr::null_mut();
        }
        self.finish_clear(release);
        panics.finish(Ok(()));
    }
    fn finish_clear(&mut self, release: bool) {
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
        self.clear(true);
    }
}
