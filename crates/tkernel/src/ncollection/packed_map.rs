//! Width-specific bit blocks retaining OCCT bucket and within-block traversal order.
use super::hash_policy::next_map_prime;
use std::marker::PhantomData;
mod integer;
mod iteration;
mod legacy_algebra;
mod node;
mod node_cursor;
mod object_storage;
mod protected_helpers;
pub use node::PackedMapNode;
mod queries;
mod storage;
pub use integer::{PackedInteger, PackedWord};
pub use iteration::PackedMapIter;
use storage::PackedNodeTable;
mod cursor;
pub use cursor::PackedCursor;
/// Packed integral set; `PackedMap::<i32>::default()` stores 32 integers per bit block.
pub struct PackedMap<T: PackedInteger> {
    buckets: PackedNodeTable<T>,
    bucket_count: usize,
    block_count: usize,
    count: usize,
    integer: PhantomData<T>,
}
impl<T: PackedInteger> Default for PackedMap<T> {
    fn default() -> Self {
        Self::new(1)
    }
}
impl<T: PackedInteger> PackedMap<T> {
    /// Native block-width selection; `PackedMap::<i64>::IS_64_BIT`.
    pub const IS_64_BIT: bool = T::IS_64_BIT;
    /// Number of bits in a block; `PackedMap::<i32>::BITS_PER_BLOCK == 32`.
    pub const BITS_PER_BLOCK: i32 = if T::IS_64_BIT { 64 } else { 32 };
    const SHIFT: u32 = if T::IS_64_BIT { 6 } else { 5 };
    /// Retain requested initial bucket count until storage is allocated; `PackedMap::<i32>::new(100)`.
    pub fn new(bucket_count: usize) -> Self {
        Self {
            buckets: PackedNodeTable::default(),
            bucket_count: bucket_count.max(1),
            block_count: 0,
            count: 0,
            integer: PhantomData,
        }
    }
    /// The signed constructor: counts below one select a single bucket, never an error;
    /// `PackedMap::<i32>::from_int(-5).bucket_count() == 1`.
    pub fn from_int(bucket_count: i32) -> Self {
        Self::new(usize::try_from(bucket_count).unwrap_or(0))
    }
    /// Signed `ReSize`: a negative request behaves as zero; `map.resize_int(-1)`.
    pub fn resize_int(&mut self, requested: i32) {
        self.resize(usize::try_from(requested).unwrap_or(0));
    }
    /// Count stored values; `map.len()`.
    pub fn len(&self) -> usize {
        self.count
    }
    /// Return OCCT's signed extent; `map.extent()`.
    pub fn extent(&self) -> i32 {
        self.count as i32
    }
    /// Check whether any blocks remain; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.block_count == 0
    }
    /// Inspect the hash modulus; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        self.bucket_count
    }
    /// Release storage while retaining the hash modulus; `map.clear()`.
    pub fn clear(&mut self) {
        self.buckets = PackedNodeTable::default();
        self.count = 0;
        self.block_count = 0;
    }
    /// Rehash by the native prime-growth table and reverse each new chain; `map.resize(1000)`.
    pub fn resize(&mut self, requested: usize) {
        let modulus: usize = next_map_prime(requested).max(self.bucket_count);
        if modulus == self.bucket_count && !self.is_empty() {
            return;
        }
        let buckets = self.buckets.rehashed(modulus);
        self.bucket_count = modulus;
        self.buckets = buckets;
    }
    /// Add a bit, growing based on occupied blocks; `map.add(42)`.
    pub fn add(&mut self, value: T) -> bool {
        if self.resizable() {
            self.resize(self.block_count);
        }
        let (index, bucket) = self.location(value);
        if let Some(node) = self.buckets.find_mut(bucket, index) {
            if !node.add_value(value) {
                return false;
            }
        } else {
            self.buckets.insert_value(bucket, value);
            self.block_count += 1;
        }
        self.count += 1;
        true
    }
    /// Check one bit without allocation; `map.contains(42)`.
    pub fn contains(&self, value: T) -> bool {
        if self.is_empty() {
            return false;
        }
        let (index, bucket) = self.location(value);
        self.buckets
            .find(bucket, index)
            .is_some_and(|node| node.has_value(value))
    }
    /// Remove a bit and destroy empty blocks; `map.remove(42)`.
    pub fn remove(&mut self, value: T) -> bool {
        if self.is_empty() {
            return false;
        }
        let (index, bucket) = self.location(value);
        let (removed, destroyed) = self.buckets.remove_value(bucket, index, value);
        self.count -= usize::from(removed);
        self.block_count -= usize::from(destroyed);
        removed
    }
    /// Copy source blocks into destination buckets; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.clear();
        if source.is_empty() {
            return self;
        }
        self.resize(source.block_count);
        for node in source.buckets.nodes() {
            let bucket = node.hash_code(self.bucket_count);
            self.buckets.insert_parts(bucket, node.mask(), node.bits());
            self.block_count += 1;
        }
        self.count = source.count;
        self
    }
    /// Transfer buckets and reset the source modulus to one; `PackedMap::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// Replace contents with moved source storage; `map.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) -> &mut Self {
        self.clear();
        *self = Self::move_from(source);
        self
    }
    fn location(&self, value: T) -> (T::Word, usize) {
        let index: T::Word = Self::packed_key_index(value);
        (index, Self::hash_code(index, self.bucket_count))
    }
}
impl<T: PackedInteger> Clone for PackedMap<T> {
    fn clone(&self) -> Self {
        let mut copy: Self = Self::default();
        copy.assign(self);
        copy
    }
    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}

mod method_contracts;
mod operands;
pub use operands::PackedMapOperand;

#[cfg(test)]
mod engine_contracts;
