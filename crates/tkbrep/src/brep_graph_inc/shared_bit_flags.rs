// Rust translation of ModelingData/TKBRep/BRepGraphInc/BRepGraphInc_BitFlags.hxx
// (the OwnedFlags plane of the stores) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A `BRepGraphInc_BitFlags` plane whose bits change through a shared reference.
//!
//! `BRepGraph_LayerLock` sets and clears the ownership bits from `const` members and
//! from its noexcept callbacks, which natively write through the attached
//! `BRepGraph*`. The Rust layer only ever holds `&BRepGraph` (docs/topology-design.md
//! section 11.10), so the owned plane keeps its blocks in atomics: the other planes
//! stay plain [`BitFlags`](super::BitFlags).
use std::sync::atomic::{AtomicU64, Ordering};

const BITS_PER_BLOCK: usize = 64;

/// A dense bit vector with the `BRepGraphInc_BitFlags` layout whose `set`, `clear` and
/// `test` take `&self`.
///
/// ```
/// use rustycad_tkbrep::brep_graph_inc::SharedBitFlags;
/// let mut flags = SharedBitFlags::new();
/// flags.resize(70);
/// flags.set(65);
/// assert!(flags.test(65) && !flags.test(64));
/// flags.clear(65);
/// assert!(!flags.has_any_bit_set());
/// ```
#[derive(Debug, Default)]
pub struct SharedBitFlags {
    blocks: Vec<AtomicU64>,
    bit_count: usize,
}

impl SharedBitFlags {
    /// `BRepGraphInc_BitFlags()`: empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Resize(theCount)`: new bits are false; bits past `count` are masked off.
    pub fn resize(&mut self, count: usize) {
        let block_count = count.div_ceil(BITS_PER_BLOCK);
        self.blocks.resize_with(block_count, || AtomicU64::new(0));
        self.bit_count = count;
        let tail = count % BITS_PER_BLOCK;
        if let (Some(last), true) = (self.blocks.last_mut(), tail != 0) {
            *last.get_mut() &= (1_u64 << tail) - 1;
        }
    }

    /// `Set(theIndex)`.
    pub fn set(&self, index: u32) {
        let (block, mask) = self.locate(index);
        self.blocks[block].fetch_or(mask, Ordering::AcqRel);
    }

    /// `Clear(theIndex)`.
    pub fn clear(&self, index: u32) {
        let (block, mask) = self.locate(index);
        self.blocks[block].fetch_and(!mask, Ordering::AcqRel);
    }

    /// `Test(theIndex)`.
    pub fn test(&self, index: u32) -> bool {
        let (block, mask) = self.locate(index);
        self.blocks[block].load(Ordering::Acquire) & mask != 0
    }

    /// `ClearAll()`.
    pub fn clear_all(&mut self) {
        for block in &mut self.blocks {
            *block.get_mut() = 0;
        }
    }

    /// `HasAnyBitSet()`.
    pub fn has_any_bit_set(&self) -> bool {
        self.blocks
            .iter()
            .any(|block| block.load(Ordering::Acquire) != 0)
    }

    /// `BitCount()`.
    pub fn bit_count(&self) -> usize {
        self.bit_count
    }

    // The block and the bit mask of an index; an index past the blocks panics, as the
    // plain plane does (natively an unchecked read).
    fn locate(&self, index: u32) -> (usize, u64) {
        let block = index as usize / BITS_PER_BLOCK;
        assert!(
            block < self.blocks.len(),
            "BRepGraphInc_BitFlags: index {index} beyond {} bits",
            self.blocks.len() * BITS_PER_BLOCK
        );
        (block, 1_u64 << (index as usize % BITS_PER_BLOCK))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resize_keeps_bits_and_masks_the_tail() {
        let mut flags = SharedBitFlags::new();
        flags.resize(70);
        flags.set(3);
        flags.set(69);
        flags.resize(65);
        assert!(flags.test(3) && !flags.test(64));
        assert_eq!(flags.bit_count(), 65);
        flags.resize(70);
        assert!(!flags.test(69));
        flags.clear_all();
        assert!(!flags.has_any_bit_set());
    }

    #[test]
    #[should_panic(expected = "index 64")]
    fn out_of_block_index_panics() {
        let mut flags = SharedBitFlags::new();
        flags.resize(10);
        flags.set(64);
    }
}
