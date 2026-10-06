//! Width-specific bit blocks retaining OCCT bucket and within-block traversal order.
use super::hash_policy::next_map_prime;
use std::marker::PhantomData;
mod integer;
mod iteration;
mod legacy_algebra;
mod node;
mod node_cursor;
mod protected_helpers;
pub use node::PackedMapNode;
mod queries;
pub use integer::{PackedInteger, PackedWord};
pub use iteration::PackedMapIter;
mod cursor;
pub use cursor::PackedCursor;
#[derive(Clone, Copy)]
struct PackedBlock<W: PackedWord> {
    index: W,
    bits: W,
}
/// Packed integral set; `PackedMap::<i32>::default()` stores 32 integers per bit block.
pub struct PackedMap<T: PackedInteger> {
    buckets: Vec<Vec<PackedBlock<T::Word>>>,
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
            buckets: Vec::new(),
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
        self.buckets.clear();
        self.count = 0;
        self.block_count = 0;
    }
    /// Rehash by the native prime-growth table and reverse each new chain; `map.resize(1000)`.
    pub fn resize(&mut self, requested: usize) {
        let modulus: usize = next_map_prime(requested).max(self.bucket_count);
        if modulus == self.bucket_count && !self.is_empty() {
            return;
        }
        let mut buckets: Vec<Vec<PackedBlock<T::Word>>> =
            (0..=modulus).map(|_| Vec::new()).collect();
        for bucket in self.buckets.drain(..) {
            for block in bucket {
                buckets[Self::hash_code(block.index, modulus)].insert(0, block);
            }
        }
        self.bucket_count = modulus;
        self.buckets = buckets;
    }
    /// Add a bit, growing based on occupied blocks; `map.add(42)`.
    pub fn add(&mut self, value: T) -> bool {
        if self.resizable() {
            self.resize(self.block_count);
        }
        let (index, bit, bucket) = self.location(value);
        if let Some(block) = self.buckets[bucket]
            .iter_mut()
            .find(|block| block.index.bits() == index)
        {
            if block.bits.bits() & bit != 0 {
                return false;
            }
            block.bits = T::Word::from_bits(block.bits.bits() | bit);
        } else {
            self.buckets[bucket].insert(
                0,
                PackedBlock {
                    index: T::Word::from_bits(index),
                    bits: T::Word::from_bits(bit),
                },
            );
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
        let (index, bit, bucket) = self.location(value);
        self.buckets[bucket]
            .iter()
            .any(|block| block.index.bits() == index && block.bits.bits() & bit != 0)
    }
    /// Remove a bit and destroy empty blocks; `map.remove(42)`.
    pub fn remove(&mut self, value: T) -> bool {
        if self.is_empty() {
            return false;
        }
        let (index, bit, bucket) = self.location(value);
        let Some(position) = self.buckets[bucket]
            .iter()
            .position(|block| block.index.bits() == index)
        else {
            return false;
        };
        let block: &mut PackedBlock<T::Word> = &mut self.buckets[bucket][position];
        if block.bits.bits() & bit == 0 {
            return false;
        }
        block.bits = T::Word::from_bits(block.bits.bits() & !bit);
        if block.bits.bits() == 0 {
            self.buckets[bucket].remove(position);
            self.block_count -= 1;
        }
        self.count -= 1;
        true
    }
    /// Copy source blocks into destination buckets; `map.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.clear();
        if source.is_empty() {
            return self;
        }
        self.resize(source.block_count);
        for bucket in &source.buckets {
            for block in bucket {
                self.buckets[Self::hash_code(block.index, self.bucket_count)].insert(0, *block);
            }
        }
        self.count = source.count;
        self.block_count = source.block_count;
        self
    }
    /// Transfer buckets and reset the source modulus to one; `PackedMap::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// Replace contents with moved source storage; `map.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) -> &mut Self {
        *self = Self::move_from(source);
        self
    }
    fn location(&self, value: T) -> (u64, u64, usize) {
        let index: T::Word = Self::packed_key_index(value);
        let bit: u64 = 1 << (value.bits() & (Self::BITS_PER_BLOCK as u64 - 1));
        (index.bits(), bit, Self::hash_code(index, self.bucket_count))
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
