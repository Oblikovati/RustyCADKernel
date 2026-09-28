// Copyright (c) 2005-2026 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Width-specific bit blocks retaining OCCT bucket and within-block traversal order.
use super::hash_policy::next_map_prime;
use std::marker::PhantomData;
mod integer;
mod iteration;
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
                buckets[block.index.bits() as usize % modulus + 1].insert(0, block);
            }
        }
        self.bucket_count = modulus;
        self.buckets = buckets;
    }
    /// Add a bit, growing based on occupied blocks; `map.add(42)`.
    pub fn add(&mut self, value: T) -> bool {
        if self.is_empty() || self.block_count > self.bucket_count {
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
    pub fn assign(&mut self, source: &Self) {
        self.clear();
        if source.is_empty() {
            return;
        }
        self.resize(source.block_count);
        for bucket in &source.buckets {
            for block in bucket {
                self.buckets[block.index.bits() as usize % self.bucket_count + 1].insert(0, *block);
            }
        }
        self.count = source.count;
        self.block_count = source.block_count;
    }
    /// Transfer buckets and reset the source modulus to one; `PackedMap::move_from(&mut source)`.
    pub fn move_from(source: &mut Self) -> Self {
        std::mem::take(source)
    }
    /// Replace contents with moved source storage; `map.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = Self::move_from(source);
    }
    fn location(&self, value: T) -> (u64, u64, usize) {
        let index: u64 = value.bits() >> Self::SHIFT;
        let bit: u64 = 1 << (value.bits() & (Self::BITS_PER_BLOCK as u64 - 1));
        (index, bit, index as usize % self.bucket_count + 1)
    }
}
impl<T: PackedInteger> Clone for PackedMap<T> {
    fn clone(&self) -> Self {
        let mut copy: Self = Self::default();
        copy.assign(self);
        copy
    }
}
