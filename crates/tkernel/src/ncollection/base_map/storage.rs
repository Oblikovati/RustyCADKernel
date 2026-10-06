//! Native bucket ownership for typed DataMap, Map and extension maps; other families migrate separately.
#![allow(unsafe_code)]
use super::{BaseMap, BaseMapResize, MapLinkNode, bucket_table::BucketTable};
use crate::{ncollection::BaseAllocator, standard::Failure};

/// Owns the `NCollection_BaseMap` bucket table and allocator handle, but never node objects.
/// Example: `let map = BaseMapStorage::new(101, None); assert!(map.is_empty());`.
/// Custom maps must destroy their nodes explicitly before this value drops.
pub struct BaseMapStorage {
    pub(super) allocator: BaseAllocator,
    pub(super) table: Option<BucketTable>,
    pub(super) bucket_count: usize,
    pub(super) size: usize,
}

impl BaseMapStorage {
    /// Creates unallocated storage, selecting the common allocator for null; `BaseMapStorage::new(5, None)`.
    pub fn new(bucket_count: usize, allocator: Option<BaseAllocator>) -> Self {
        Self {
            allocator: allocator.unwrap_or_else(BaseAllocator::common),
            table: None,
            bucket_count,
            size: 0,
        }
    }
    /// Preserves the native ignored legacy bool argument; `BaseMapStorage::new_legacy(5, false, None)`.
    pub fn new_legacy(
        bucket_count: usize,
        _legacy: bool,
        allocator: Option<BaseAllocator>,
    ) -> Self {
        Self::new(bucket_count, allocator)
    }
    /// Steals storage while retaining the donor allocator and resetting its modulus; `BaseMapStorage::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        Self {
            allocator: source.allocator.clone(),
            table: source.table.take(),
            bucket_count: std::mem::take(&mut source.bucket_count),
            size: std::mem::take(&mut source.size),
        }
    }
    /// Returns native const `Buckets`, null before allocation or after release; `map.buckets()`.
    pub fn buckets(&self) -> *const *mut MapLinkNode {
        self.table
            .as_ref()
            .map_or(std::ptr::null(), |table| table.as_ptr().cast_const())
    }
    /// Returns native mutable `Buckets`; `map.buckets_mut()`.
    pub fn buckets_mut(&mut self) -> *mut *mut MapLinkNode {
        self.table
            .as_ref()
            .map_or(std::ptr::null_mut(), BucketTable::as_ptr)
    }
    /// Borrows the first table's initialized node pointers; `map.bucket_slots()[0]`.
    pub fn bucket_slots(&self) -> &[*mut MapLinkNode] {
        self.table
            .as_ref()
            .map_or(&[], |table| &table.as_slice()[..=self.bucket_count])
    }
    /// Changes node pointers without taking ownership of nodes; `map.bucket_slots_mut()[0] = node`.
    pub fn bucket_slots_mut(&mut self) -> &mut [*mut MapLinkNode] {
        self.table.as_mut().map_or(&mut [], |table| {
            &mut table.as_mut_slice()[..=self.bucket_count]
        })
    }
    /// Allocates a zeroed replacement when native `beginResize` would; `map.begin_resize(100)?`.
    pub fn begin_resize(&self, extent: usize) -> Result<BaseMapResize, Failure> {
        BaseMapResize::prepare(extent, self.bucket_count, self.table.is_some(), 1)
    }
    /// Adopts a pending resize; no-op plans preserve this table; `map.end_resize(plan)`.
    pub fn end_resize(&mut self, mut plan: BaseMapResize) {
        if let Some(table) = plan.replacement.take() {
            self.table = Some(table);
            self.bucket_count = plan.bucket_count;
        }
    }
    /// Reports native `resizable`: empty or size greater than modulus; `map.resizable()`.
    pub fn resizable(&self) -> bool {
        self.size == 0 || self.size > self.bucket_count
    }
    /// Performs native unsigned count increment; `assert_eq!(map.increment(), 1)`.
    pub fn increment(&mut self) -> usize {
        self.size = self.size.wrapping_add(1);
        self.size
    }
    /// Performs native unsigned count decrement; `map.decrement()`.
    pub fn decrement(&mut self) -> usize {
        self.size = self.size.wrapping_sub(1);
        self.size
    }
    /// Exchanges every base field, including allocator identity; `first.exchange(&mut second)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// Calls the destructor in bucket/chain order, optionally releasing the table; `unsafe { map.destroy(drop_node, true) }`.
    ///
    /// # Safety
    /// When size is nonzero, every bucket chain must consist of valid, readable, acyclic
    /// `MapLinkNode` objects. The callback must not invalidate other unvisited nodes or links.
    pub unsafe fn destroy(
        &mut self,
        mut delete_node: impl FnMut(*mut MapLinkNode, &mut BaseAllocator),
        release: bool,
    ) {
        if self.size != 0 {
            // SAFETY: the caller guarantees that all nonempty bucket chains remain valid.
            unsafe { self.destroy_nonempty(&mut delete_node) };
        }
        self.finish_destroy(release);
    }
    // Shared final phase: callers have completed all node callbacks and cleared bucket heads.
    pub(in crate::ncollection) fn finish_destroy(&mut self, release: bool) {
        self.size = 0;
        if release {
            self.table = None;
        }
    }
    unsafe fn destroy_nonempty(
        &mut self,
        delete_node: &mut impl FnMut(*mut MapLinkNode, &mut BaseAllocator),
    ) {
        let table = self
            .table
            .as_mut()
            .expect("BaseMap nonzero size requires allocated bucket storage");
        for first in &mut table.as_mut_slice()[..=self.bucket_count] {
            // SAFETY: destroy's caller guarantees all chains remain readable through this walk.
            unsafe { destroy_chain(*first, &mut self.allocator, delete_node) };
            *first = std::ptr::null_mut();
        }
    }
}

impl BaseMap for BaseMapStorage {
    fn nb_buckets(&self) -> usize {
        self.bucket_count
    }
    fn size(&self) -> usize {
        self.size
    }
    fn allocator(&self) -> &BaseAllocator {
        &self.allocator
    }
}

unsafe fn destroy_chain(
    mut node: *mut MapLinkNode,
    allocator: &mut BaseAllocator,
    delete_node: &mut impl FnMut(*mut MapLinkNode, &mut BaseAllocator),
) {
    while !node.is_null() {
        // SAFETY: caller guarantees the current link is live; cache before callback may free it.
        let next = unsafe { (*node).next() };
        delete_node(node, allocator);
        node = next;
    }
}
