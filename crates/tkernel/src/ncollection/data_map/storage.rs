//! Sole ownership of real BaseMapStorage tables and native DataMapNode chains.
use crate::ncollection::{BaseAllocator, BaseMap, BaseMapStorage, DataMapNode, MapLinkNode};
use std::{marker::PhantomData, panic::RefUnwindSafe};
mod resize;
mod topology;
pub(super) struct DataMapStorage<K, V> {
    pub(super) base: BaseMapStorage,
    ownership: PhantomData<Box<(K, V)>>,
}
// SAFETY: this private storage exclusively owns all reachable nodes. Mutations require &mut;
// shared queries only read keys/values/links. The allocator is thread-safe, and ownership moves
// the entire graph. Erased BaseMapStorage and public non-owning MapLinkNode remain !Send/!Sync.
unsafe impl<K: Send, V: Send> Send for DataMapStorage<K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for DataMapStorage<K, V> {}
impl<K: RefUnwindSafe, V: RefUnwindSafe> std::panic::UnwindSafe for DataMapStorage<K, V> {}
impl<K: RefUnwindSafe, V: RefUnwindSafe> RefUnwindSafe for DataMapStorage<K, V> {}
impl<K, V> DataMapStorage<K, V> {
    pub(super) fn new(buckets: usize, allocator: BaseAllocator) -> Self {
        // Native constructors retain even zero buckets until the first insertion resizes.
        Self {
            base: BaseMapStorage::new(buckets, Some(allocator)),
            ownership: PhantomData,
        }
    }
    pub(super) unsafe fn node(&self, pointer: *mut MapLinkNode) -> &DataMapNode<K, V> {
        unsafe { &*pointer.cast::<DataMapNode<K, V>>() }
    }
    pub(super) unsafe fn node_mut(&mut self, pointer: *mut MapLinkNode) -> &mut DataMapNode<K, V> {
        unsafe { &mut *pointer.cast::<DataMapNode<K, V>>() }
    }
    pub(super) fn clear_with_allocator(&mut self, requested: Option<BaseAllocator>) {
        let release = requested.as_ref() != Some(self.base.allocator());
        self.clear(release);
        if release {
            self.base = BaseMapStorage::new(self.base.nb_buckets(), requested);
        }
    }
}
impl<K, V> Drop for DataMapStorage<K, V> {
    fn drop(&mut self) {
        self.clear(true);
    }
}
