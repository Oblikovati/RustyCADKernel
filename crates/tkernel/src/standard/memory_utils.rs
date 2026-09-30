// Rust translation of FoundationClasses/TKernel/Standard/Standard_MemoryUtils.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `opencascade::make_shared` / `make_oshared` / `make_unique`: owning pointers whose
//! storage comes from OCCT allocators.
//!
//! C++ forwards constructor arguments; Rust constructs the value first and moves it in.
//! `std::shared_ptr` becomes `Arc` and `std::unique_ptr` becomes `Box`. The global
//! `NCollection_Allocator` is Rust's global allocator (the NATIVE `Standard::Allocate`).
use crate::ncollection::{AllocatedBox, AllocatorVector, BaseAllocator, OccAllocator};
use std::ops::Deref;
use std::sync::Arc;

/// Shared ownership through the global allocator; `*make_shared(5) == 5`.
pub fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

/// Unique ownership through the global allocator; `*make_unique(String::new()) == ""`.
pub fn make_unique<T>(value: T) -> Box<T> {
    Box::new(value)
}

/// Shared ownership of a value stored by `allocator` (the `handle<NCollection_BaseAllocator>`
/// overload); `*make_oshared(&BaseAllocator::incremental(1024), 7) == 7`. The value lives in
/// the allocator's memory; the reference count lives beside it in the global heap.
pub fn make_oshared<T>(allocator: &BaseAllocator, value: T) -> Arc<AllocatedBox<T>> {
    Arc::new(allocator.allocate_value(value))
}

/// Shared ownership of a value stored through a typed allocator view (both
/// `NCollection_OccAllocator<T>` overloads: a borrowed allocator is copied, a moved one is
/// consumed, which Rust expresses the same way); `*make_oshared_in(&OccAllocator::default(), 1)`.
pub fn make_oshared_in<T>(allocator: &OccAllocator<T>, value: T) -> OcctShared<T> {
    let mut storage = allocator.vector();
    storage.push(value);
    OcctShared {
        storage: Arc::new(storage),
    }
}

/// A shared value kept in allocator-owned storage; `*shared == value`.
pub struct OcctShared<T> {
    storage: Arc<AllocatorVector<T>>,
}
impl<T> Deref for OcctShared<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.storage[0]
    }
}
impl<T> Clone for OcctShared<T> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }
}
impl<T> OcctShared<T> {
    /// Number of owners, like `shared_ptr::use_count`; `shared.use_count() == 1`.
    pub fn use_count(&self) -> usize {
        Arc::strong_count(&self.storage)
    }
}
