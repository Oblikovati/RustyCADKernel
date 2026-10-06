//! Sole ownership of real native bucket tables and sealed typed node chains.
#![allow(unsafe_code)]
use crate::ncollection::{BaseAllocator, BaseMap, BaseMapStorage, MapLinkNode};
use std::{marker::PhantomData, panic::RefUnwindSafe};
pub(in crate::ncollection) mod node_contract;
pub(super) use node_contract::CanonicalMapNode;
mod lookup;
mod range;
pub(super) use range::BucketNodeRange;
mod position;
pub(super) use position::BucketChainPosition;
mod resize;
mod topology;
pub(super) struct OwnedBucketChains<N: CanonicalMapNode> {
    pub(super) base: BaseMapStorage,
    ownership: PhantomData<Box<N>>,
}
// SAFETY: this private storage exclusively owns all reachable nodes. Mutations require &mut;
// shared queries only read keys/values/links. The allocator is thread-safe, and ownership moves
// the entire graph. The sealed contract accounts for every owned payload through Key/Value.
// Erased BaseMapStorage and public non-owning linked nodes remain !Send/!Sync.
unsafe impl<N: CanonicalMapNode> Send for OwnedBucketChains<N>
where
    N::Key: Send,
    N::Value: Send,
{
}
unsafe impl<N: CanonicalMapNode> Sync for OwnedBucketChains<N>
where
    N::Key: Sync,
    N::Value: Sync,
{
}
impl<N: CanonicalMapNode> std::panic::UnwindSafe for OwnedBucketChains<N>
where
    N::Key: RefUnwindSafe,
    N::Value: RefUnwindSafe,
{
}
impl<N: CanonicalMapNode> RefUnwindSafe for OwnedBucketChains<N>
where
    N::Key: RefUnwindSafe,
    N::Value: RefUnwindSafe,
{
}
impl<N: CanonicalMapNode> OwnedBucketChains<N> {
    pub(super) fn new(buckets: usize, allocator: BaseAllocator) -> Self {
        // Native constructors retain even zero buckets until the first insertion resizes.
        Self {
            base: BaseMapStorage::new(buckets, Some(allocator)),
            ownership: PhantomData,
        }
    }
    pub(super) unsafe fn node(&self, pointer: *mut MapLinkNode) -> &N {
        unsafe { &*pointer.cast::<N>() }
    }
    pub(super) unsafe fn node_mut(&mut self, pointer: *mut MapLinkNode) -> &mut N {
        unsafe { &mut *pointer.cast::<N>() }
    }
    pub(super) fn clear_with_allocator(&mut self, requested: Option<BaseAllocator>) {
        let release = requested.as_ref() != Some(self.base.allocator());
        self.clear(release);
        if release {
            self.base = BaseMapStorage::new(self.base.nb_buckets(), requested);
        }
    }
}
impl<N: CanonicalMapNode> Drop for OwnedBucketChains<N> {
    fn drop(&mut self) {
        self.clear(true);
    }
}
