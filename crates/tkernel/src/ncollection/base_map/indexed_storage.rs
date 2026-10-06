//! The index-to-node table is allocated independently of BaseMap's hash table.
use super::{
    BaseMap, BaseMapStorage, MapLinkNode, bucket_table::BucketTable, resize_plan::table_slot_count,
};
use crate::{ncollection::BaseAllocator, standard::Failure};

/// Extension storage for native `NCollection_BaseIndexedMap` subclasses.
/// Example: `let mut map = BaseIndexedMapStorage::new(101, None); map.resize_index_table(101)?;`.
pub struct BaseIndexedMapStorage {
    base: BaseMapStorage,
    indices: Option<BucketTable>,
}

impl BaseIndexedMapStorage {
    /// Creates unallocated hash and index tables; `BaseIndexedMapStorage::new(101, None)`.
    pub fn new(bucket_count: usize, allocator: Option<BaseAllocator>) -> Self {
        Self {
            base: BaseMapStorage::new(bucket_count, allocator),
            indices: None,
        }
    }
    /// Borrows common native state; `map.base().nb_buckets()`.
    pub fn base(&self) -> &BaseMapStorage {
        &self.base
    }
    /// Allows the native hash-table resize sequence; `map.base_mut().end_resize(plan)`.
    pub fn base_mut(&mut self) -> &mut BaseMapStorage {
        &mut self.base
    }
    /// Reallocates the index table, retaining the native prefix only when hash storage exists; `map.resize_index_table(1009)?`.
    pub fn resize_index_table(&mut self, bucket_count: usize) -> Result<(), Failure> {
        let count = table_slot_count(bucket_count, 1)?;
        let old_count = if self.base.table.is_some() && self.indices.is_some() {
            self.base.bucket_count + 1
        } else {
            0
        };
        match &mut self.indices {
            Some(table) => table.resize(count)?,
            None => self.indices = Some(BucketTable::zeroed(count)?),
        }
        let table = self
            .indices
            .as_mut()
            .expect("new index table was allocated");
        table.initialize_nulls(old_count.min(count));
        Ok(())
    }
    /// Returns native const index table, null when unallocated; `map.index_buckets()`.
    pub fn index_buckets(&self) -> *const *mut MapLinkNode {
        self.indices
            .as_ref()
            .map_or(std::ptr::null(), |table| table.as_ptr().cast_const())
    }
    /// Returns native mutable index table, null when unallocated; `map.index_buckets_mut()`.
    pub fn index_buckets_mut(&mut self) -> *mut *mut MapLinkNode {
        self.indices
            .as_ref()
            .map_or(std::ptr::null_mut(), BucketTable::as_ptr)
    }
    /// Borrows all currently allocated index slots; `map.index_slots()[1]`.
    pub fn index_slots(&self) -> &[*mut MapLinkNode] {
        self.indices.as_ref().map_or(&[], BucketTable::as_slice)
    }
    /// Mutates native index slots; `map.index_slots_mut()[1] = node`.
    pub fn index_slots_mut(&mut self) -> &mut [*mut MapLinkNode] {
        self.indices
            .as_mut()
            .map_or(&mut [], BucketTable::as_mut_slice)
    }
    /// Clears indices using the committed hash modulus, optionally releasing; `map.clear_index_table(false)`.
    /// A resize must commit the matching hash modulus before clearing, as native callers require.
    pub fn clear_index_table(&mut self, release: bool) {
        let Some(table) = &mut self.indices else {
            return;
        };
        table
            .as_mut_slice()
            .get_mut(..=self.base.bucket_count)
            .expect("BaseIndexedMap clear requires index storage for the committed hash modulus")
            .fill(std::ptr::null_mut());
        if release {
            self.indices = None;
        }
    }
    /// Exchanges both allocations, counts, and allocator identity; `first.exchange(&mut second)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
}

impl BaseMap for BaseIndexedMapStorage {
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
