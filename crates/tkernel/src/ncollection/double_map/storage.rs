//! One canonical contiguous pair of bucket tables owns every real dual-chain node exactly once.
#![allow(unsafe_code)]
use crate::ncollection::{
    BaseAllocator, BaseDoubleMapStorage, BaseMap, DoubleMapNode, MapLinkNode,
};
use std::{marker::PhantomData, panic::RefUnwindSafe};
mod topology;
use topology::DoubleTopology;
pub use topology::{DoubleMapObservation, DoubleMapObserver};
mod lifecycle;
mod links;
pub(super) use links::UnlinkedDoubleNode;
mod resize;
pub(super) struct DoubleStorage<A, B> {
    topology: DoubleTopology,
    ownership: PhantomData<(Box<A>, Box<B>)>,
}
// SAFETY: this private owner exclusively owns both-chain nodes, with all mutations requiring &mut.
// Observers copy metadata only. Payloads never move when the whole owning graph transfers threads.
// Erased storage plus owned payload markers preserve key covariance despite node's invariant Next2.
unsafe impl<A: Send, B: Send> Send for DoubleStorage<A, B> {}
unsafe impl<A: Sync, B: Sync> Sync for DoubleStorage<A, B> {}
impl<A: RefUnwindSafe, B: RefUnwindSafe> std::panic::UnwindSafe for DoubleStorage<A, B> {}
impl<A: RefUnwindSafe, B: RefUnwindSafe> RefUnwindSafe for DoubleStorage<A, B> {}
impl<A, B> DoubleStorage<A, B> {
    pub(super) fn new(count: usize, allocator: BaseAllocator) -> Self {
        Self {
            topology: DoubleTopology::new(count, allocator),
            ownership: PhantomData,
        }
    }
    pub(super) fn base(&self) -> &BaseDoubleMapStorage {
        self.topology.read()
    }
    pub(super) fn len(&self) -> usize {
        self.base().size()
    }
    pub(super) fn bucket_count(&self) -> usize {
        self.base().nb_buckets()
    }
    pub(super) fn allocator(&self) -> &BaseAllocator {
        self.base().allocator()
    }
    pub(super) fn observer(&self) -> DoubleMapObserver {
        self.topology.observer()
    }
    pub(super) fn exchange(&mut self, other: &mut Self) {
        self.topology.exchange(&mut other.topology);
    }
    pub(super) fn head(&self, bucket: usize, secondary: bool) -> *mut MapLinkNode {
        if secondary {
            self.base().second_slots()[bucket]
        } else {
            self.base().bucket_slots()[bucket]
        }
    }
    pub(super) unsafe fn node(&self, pointer: *mut MapLinkNode) -> &DoubleMapNode<A, B> {
        // SAFETY: internal callers supply a live node reached from one of this owner's two chains.
        unsafe { &*pointer.cast::<DoubleMapNode<A, B>>() }
    }
    pub(super) fn find(
        &self,
        bucket: usize,
        secondary: bool,
        accept: impl Fn(&DoubleMapNode<A, B>) -> bool,
    ) -> Option<*mut MapLinkNode> {
        let mut node = self.head(bucket, secondary);
        while !node.is_null() {
            if accept(unsafe { self.node(node) }) {
                return Some(node);
            }
            node = unsafe { Self::next(node, secondary) };
        }
        None
    }
    pub(super) fn insert(
        &mut self,
        first: usize,
        second: usize,
        key: impl FnOnce() -> A,
        value: impl FnOnce() -> B,
    ) {
        let owner = DoubleMapNode::allocate_with(
            self.allocator(),
            self.head(first, false),
            self.head(second, true),
            key,
            value,
        );
        let (node, _allocator) = DoubleMapNode::into_raw(owner);
        self.publish_node(first, second, node);
    }
    fn publish_node(&mut self, first: usize, second: usize, node: *mut MapLinkNode) {
        self.topology.edit(|base| {
            base.bucket_slots_mut()[first] = node;
            base.second_slots_mut()[second] = node;
            base.increment();
        });
    }
}
impl<A, B> Drop for DoubleStorage<A, B> {
    fn drop(&mut self) {
        self.clear(true);
    }
}
