//! Synchronized adapter for opt-in parallel BVH construction.
use super::bounding_box::BvhBox;
use super::set::{BvhSet, BvhSharedSet};
use super::vector::BvhVector;
use rustycad_tkernel::standard::Failure;
use std::sync::{Mutex, PoisonError};

/// Gives a mutable set synchronized read/swap access for parallel builders.
/// Example: `let shared = BvhMutexSet::new(box_set);`.
pub struct BvhMutexSet<V: BvhVector, S: BvhSet<V>> {
    inner: Mutex<S>,
    vector: std::marker::PhantomData<V>,
}

impl<V: BvhVector, S: BvhSet<V>> BvhMutexSet<V, S> {
    /// Wraps `set`; `let shared = BvhMutexSet::new(set);`.
    pub fn new(set: S) -> Self {
        Self {
            inner: Mutex::new(set),
            vector: std::marker::PhantomData,
        }
    }

    /// Returns the set after all workers finish; `let set = shared.into_inner()?`.
    pub fn into_inner(self) -> Result<S, Failure> {
        self.inner
            .into_inner()
            .map_err(|_| Failure::new("BVH shared set lock poisoned"))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, S> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<V: BvhVector, S: BvhSet<V>> BvhSharedSet<V> for BvhMutexSet<V, S> {
    fn size(&self) -> i32 {
        self.lock().size()
    }

    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        self.lock().element_box(index)
    }

    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        self.lock().center(index, axis)
    }

    fn swap(&self, first: i32, second: i32) -> Result<(), Failure> {
        self.lock().swap(first, second)
    }
}

pub(crate) struct BvhBorrowedMutexSet<'a, V: BvhVector> {
    inner: Mutex<&'a mut dyn BvhSet<V>>,
}

impl<'a, V: BvhVector> BvhBorrowedMutexSet<'a, V> {
    pub(crate) fn new(set: &'a mut dyn BvhSet<V>) -> Self {
        Self {
            inner: Mutex::new(set),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, &'a mut dyn BvhSet<V>> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<V: BvhVector> BvhSharedSet<V> for BvhBorrowedMutexSet<'_, V> {
    fn size(&self) -> i32 {
        self.lock().size()
    }

    fn element_box(&self, index: i32) -> Result<BvhBox<V>, Failure> {
        self.lock().element_box(index)
    }

    fn center(&self, index: i32, axis: i32) -> Result<V::Scalar, Failure> {
        self.lock().center(index, axis)
    }

    fn swap(&self, first: i32, second: i32) -> Result<(), Failure> {
        self.lock().swap(first, second)
    }
}
