// Copyright (c) 2026 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_BitFlags.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraphInc_BitFlags`: one bit per entity index in 64-bit blocks.

const BITS_PER_BLOCK: usize = 64;

/// `BRepGraphInc_BitFlags`: a dense bit vector for per-entity boolean planes.
///
/// `set`, `clear` and `test` are unchecked natively (an index past the blocks reads out
/// of bounds); here they panic with the index and the bit count.
///
/// ```
/// use rustycad_tkbrep::brep_graph_inc::BitFlags;
/// let mut flags = BitFlags::new();
/// flags.resize(100);
/// flags.set(42);
/// assert!(flags.test(42) && flags.has_any_bit_set());
/// flags.clear(42);
/// assert!(!flags.test(42));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BitFlags {
    blocks: Vec<u64>,
    bit_count: usize,
}

impl BitFlags {
    /// `BRepGraphInc_BitFlags()`: empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Resize(theCount)`: holds `count` bits; new bits are false and bits past `count`
    /// in the last block are masked off.
    pub fn resize(&mut self, count: usize) {
        let block_count = count.div_ceil(BITS_PER_BLOCK);
        self.blocks.resize(block_count, 0);
        self.bit_count = count;
        self.mask_tail_bits();
    }

    /// `Set(theIndex)`.
    pub fn set(&mut self, index: u32) {
        let (block, mask) = self.locate(index);
        self.blocks[block] |= mask;
    }

    /// `Clear(theIndex)`.
    pub fn clear(&mut self, index: u32) {
        let (block, mask) = self.locate(index);
        self.blocks[block] &= !mask;
    }

    /// `Test(theIndex)`.
    pub fn test(&self, index: u32) -> bool {
        let (block, mask) = self.locate(index);
        self.blocks[block] & mask != 0
    }

    /// `SetAll()`: every valid bit.
    pub fn set_all(&mut self) {
        self.blocks.fill(!0);
        self.mask_tail_bits();
    }

    /// `ClearAll()`.
    pub fn clear_all(&mut self) {
        self.blocks.fill(0);
    }

    /// `HasAnyBitSet()`.
    pub fn has_any_bit_set(&self) -> bool {
        self.blocks.iter().any(|block| *block != 0)
    }

    /// `NbBlocks()`.
    pub fn nb_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// `BitCount()`.
    pub fn bit_count(&self) -> usize {
        self.bit_count
    }

    /// `IsValidIndex(theIndex)`: below the bit count.
    pub fn is_valid_index(&self, index: u32) -> bool {
        (index as usize) < self.bit_count
    }

    /// `Blocks()`: the raw blocks.
    pub fn blocks(&self) -> &[u64] {
        &self.blocks
    }

    // The block and the bit mask of an index; an index past the blocks panics.
    fn locate(&self, index: u32) -> (usize, u64) {
        let block = index as usize / BITS_PER_BLOCK;
        assert!(
            block < self.blocks.len(),
            "BRepGraphInc_BitFlags: index {index} beyond {} bits",
            self.blocks.len() * BITS_PER_BLOCK
        );
        (block, 1_u64 << (index as usize % BITS_PER_BLOCK))
    }

    fn mask_tail_bits(&mut self) {
        let tail = self.bit_count % BITS_PER_BLOCK;
        if tail == 0 {
            return;
        }
        if let Some(last) = self.blocks.last_mut() {
            *last &= (1_u64 << tail) - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_masks_the_tail_block() {
        let mut flags = BitFlags::new();
        flags.resize(70);
        flags.set_all();
        assert_eq!(flags.nb_blocks(), 2);
        assert_eq!(flags.blocks()[1], (1 << 6) - 1);
        flags.resize(65);
        assert_eq!(flags.blocks()[1], 1);
        assert!(flags.is_valid_index(64) && !flags.is_valid_index(65));
        assert_eq!(flags.bit_count(), 65);
        flags.clear_all();
        assert!(!flags.has_any_bit_set());
    }

    #[test]
    #[should_panic(expected = "index 64")]
    fn out_of_block_index_panics() {
        let mut flags = BitFlags::new();
        flags.resize(10);
        flags.set(64);
    }
}
