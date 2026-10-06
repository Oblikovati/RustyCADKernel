//! BaseDoubleMap stores both hash tables in a single Standard allocation.
#![allow(unsafe_code)]
use super::{BaseMap, BaseMapResize, BaseMapStorage, MapLinkNode};
use crate::{ncollection::BaseAllocator, standard::Failure};

/// Storage for custom `NCollection_BaseDoubleMap` implementations.
/// Example: `let map = BaseDoubleMapStorage::new(101, None); assert!(map.is_empty());`.
pub struct BaseDoubleMapStorage {
    base: BaseMapStorage,
}

/// A pending contiguous pair of bucket tables; `let plan = map.begin_resize(0)?;`.
pub struct BaseDoubleMapResize {
    plan: BaseMapResize,
}

impl BaseDoubleMapStorage {
    /// Creates two unallocated tables sharing base state; `BaseDoubleMapStorage::new(101, None)`.
    pub fn new(bucket_count: usize, allocator: Option<BaseAllocator>) -> Self {
        Self {
            base: BaseMapStorage::new(bucket_count, allocator),
        }
    }
    /// Allocates a contiguous pair when growth is needed; `map.begin_resize(100)?`.
    pub fn begin_resize(&self, extent: usize) -> Result<BaseDoubleMapResize, Failure> {
        Ok(BaseDoubleMapResize {
            plan: BaseMapResize::prepare(
                extent,
                self.base.bucket_count,
                self.base.table.is_some(),
                2,
            )?,
        })
    }
    /// Commits both pending tables together; `map.end_resize(plan)`.
    pub fn end_resize(&mut self, replacement: BaseDoubleMapResize) {
        self.base.end_resize(replacement.plan);
    }
    /// Borrows common read-only state for the legacy cursor; `BaseMapBucketIterator::new(map.base())`.
    pub fn base(&self) -> &BaseMapStorage {
        &self.base
    }
    /// Returns the first native table pointer; `map.buckets()`.
    pub fn buckets(&self) -> *const *mut MapLinkNode {
        self.base.buckets()
    }
    /// Returns the writable first table; `map.buckets_mut()`.
    pub fn buckets_mut(&mut self) -> *mut *mut MapLinkNode {
        self.base.buckets_mut()
    }
    /// Borrows first-table slots; `map.bucket_slots()[0]`.
    pub fn bucket_slots(&self) -> &[*mut MapLinkNode] {
        self.base.bucket_slots()
    }
    /// Mutates first-table links; `map.bucket_slots_mut()[0] = node`.
    pub fn bucket_slots_mut(&mut self) -> &mut [*mut MapLinkNode] {
        self.base.bucket_slots_mut()
    }
    /// Borrows the contiguous second table; `map.second_slots()[0]`.
    pub fn second_slots(&self) -> &[*mut MapLinkNode] {
        self.base
            .table
            .as_ref()
            .map_or(&[], |table| &table.as_slice()[self.base.bucket_count + 1..])
    }
    /// Mutates second-table links; `map.second_slots_mut()[0] = node`.
    pub fn second_slots_mut(&mut self) -> &mut [*mut MapLinkNode] {
        self.base.table.as_mut().map_or(&mut [], |table| {
            &mut table.as_mut_slice()[self.base.bucket_count + 1..]
        })
    }
    /// Returns native second-table pointer or null; `map.second_buckets()`.
    pub fn second_buckets(&self) -> *const *mut MapLinkNode {
        if self.base.table.is_none() {
            return std::ptr::null();
        }
        self.second_slots().as_ptr()
    }
    /// Returns the writable second-table pointer or null; `map.second_buckets_mut()`.
    pub fn second_buckets_mut(&mut self) -> *mut *mut MapLinkNode {
        if self.base.table.is_none() {
            return std::ptr::null_mut();
        }
        self.second_slots_mut().as_mut_ptr()
    }
    /// Increments the common native size; `map.increment()`.
    pub fn increment(&mut self) -> usize {
        self.base.increment()
    }
    /// Decrements the common native size; `map.decrement()`.
    pub fn decrement(&mut self) -> usize {
        self.base.decrement()
    }
    /// Tests the common native resize policy; `map.resizable()`.
    pub fn resizable(&self) -> bool {
        self.base.resizable()
    }
    /// Exchanges the contiguous allocation and all common state; `first.exchange(&mut second)`.
    pub fn exchange(&mut self, other: &mut Self) {
        self.base.exchange(&mut other.base);
    }
    pub(in crate::ncollection) fn finish_destroy(&mut self, release: bool) {
        self.second_slots_mut().fill(std::ptr::null_mut());
        self.base.finish_destroy(release);
    }
    /// Destroys through the first table and clears the second even when size is zero; `unsafe { map.destroy(drop_node, false) }`.
    ///
    /// # Safety
    /// The first table must satisfy `BaseMapStorage::destroy`'s live acyclic chain contract.
    pub unsafe fn destroy(
        &mut self,
        delete_node: impl FnMut(*mut MapLinkNode, &mut BaseAllocator),
        release: bool,
    ) {
        // SAFETY: the caller provides the same first-table chain guarantees.
        unsafe { self.base.destroy(delete_node, false) };
        self.finish_destroy(release);
    }
}

impl BaseDoubleMapResize {
    /// The computed native modulus, even on no-op requests; `plan.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.plan.bucket_count()
    }
    /// True when both replacement tables were allocated; `plan.is_resize()`.
    pub fn is_resize(&self) -> bool {
        self.plan.is_resize()
    }
    /// First-table slots for relinking; `plan.bucket_slots_mut()[0] = node`.
    pub fn bucket_slots_mut(&mut self) -> &mut [*mut MapLinkNode] {
        if !self.is_resize() {
            return &mut [];
        }
        let count = self.bucket_count() + 1;
        &mut self.plan.bucket_slots_mut()[..count]
    }
    /// Second-table slots for relinking; `plan.second_slots_mut()[0] = node`.
    pub fn second_slots_mut(&mut self) -> &mut [*mut MapLinkNode] {
        if !self.is_resize() {
            return &mut [];
        }
        let count = self.bucket_count() + 1;
        &mut self.plan.bucket_slots_mut()[count..]
    }
}

impl BaseMap for BaseDoubleMapStorage {
    fn nb_buckets(&self) -> usize {
        self.base.nb_buckets()
    }
    fn size(&self) -> usize {
        self.base.size()
    }
    fn allocator(&self) -> &BaseAllocator {
        self.base.allocator()
    }
}
