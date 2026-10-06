//! Private typed ownership over the real native table, bitmap and block allocation engine.
use super::super::SparseArrayBase;
use std::{marker::PhantomData, panic::RefUnwindSafe};
mod cloning;
mod editing;
mod layout;
mod lifecycle;
mod traversal;
pub(super) use layout::sparse_project;
use layout::sparse_stride;
use lifecycle::sparse_noop_destroy;
pub(super) use traversal::first_raw_sparse_key;

pub(super) struct TypedSparseStorage<T> {
    pub(super) base: SparseArrayBase,
    // Heap-owned values remain stationary when the owner moves; preserve unconditional Unpin.
    ownership: PhantomData<Box<T>>,
}

// SAFETY: this private owner constructs its base and never clones/exposes its Rc state. The
// strong count remains one, so moving the owner cannot race a refcount operation. All mutation
// requires &mut Self; callbacks only touch its owned T values. The common allocator is thread-safe.
// The erased public SparseArrayBase deliberately does NOT implement Send or Sync.
unsafe impl<T: Send> Send for TypedSparseStorage<T> {}
// SAFETY: shared methods only read state/bitmap and project &T; they never clone the private Rc.
unsafe impl<T: Sync> Sync for TypedSparseStorage<T> {}
// BTreeMap's previous UnwindSafe implementation required RefUnwindSafe values (Rust 1.98.1).
impl<T: RefUnwindSafe> std::panic::UnwindSafe for TypedSparseStorage<T> {}
impl<T: RefUnwindSafe> RefUnwindSafe for TypedSparseStorage<T> {}

impl<T> TypedSparseStorage<T> {
    pub(super) fn new(increment: usize) -> Self {
        Self {
            base: SparseArrayBase::new(sparse_stride::<T>(), increment, sparse_noop_destroy),
            ownership: PhantomData,
        }
    }
    fn occupied_address(&self, index: usize) -> Option<*mut std::ffi::c_void> {
        // Missing safe Option lookups must not allocate a native Failure/message just to discard it.
        self.base
            .has_value(index)
            .then(|| self.base.get_value(index).unwrap())
    }
    pub(super) fn value(&self, index: usize) -> Option<&T> {
        let address = self.occupied_address(index)?;
        // SAFETY: occupied slots contain initialized T; &self excludes safe mutation.
        Some(unsafe { &*sparse_project::<T>(address) })
    }
    pub(super) fn value_mut(&mut self, index: usize) -> Option<&mut T> {
        let address = self.occupied_address(index)?;
        // SAFETY: &mut self exclusively borrows every occupied T.
        Some(unsafe { &mut *sparse_project::<T>(address) })
    }
}
impl<T> Drop for TypedSparseStorage<T> {
    fn drop(&mut self) {
        // Typed cleanup drains objects before raw field Drop frees its now-empty byte storage.
        self.clear();
    }
}
