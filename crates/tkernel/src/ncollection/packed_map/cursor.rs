//! The native `NCollection_PackedMap::Iterator` state, read against the live map.
//!
//! OCCT code such as `Poly_MakeLoops::HeapOfInteger` keeps one iterator while it removes
//! keys from the map. The native iterator stores the current node, bucket and a mask of
//! the bits above the current key, and it reads the node bits again on every step, so a
//! removed key is skipped and the chain is followed through its live links. This cursor
//! stores the same state by value and is advanced against whatever the map holds now.
use super::{PackedInteger, PackedMap, PackedWord};
/// Bucket, node (by block index) and remaining-bit mask of a native iterator; `map.cursor_first()`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PackedCursor {
    bucket: usize,
    block: Option<u64>,
    mask: u64,
}
impl<T: PackedInteger> PackedMap<T> {
    const FULL_MASK: u64 = if T::IS_64_BIT {
        u64::MAX
    } else {
        u32::MAX as u64
    };
    /// Positions a cursor like the native iterator constructor; `let (cursor, key) = map.cursor_first();`.
    pub fn cursor_first(&self) -> (PackedCursor, Option<T>) {
        let mut cursor = PackedCursor {
            bucket: 0,
            block: None,
            mask: Self::FULL_MASK,
        };
        self.cursor_find_first(&mut cursor);
        let key = self.cursor_next(&mut cursor);
        (cursor, key)
    }
    /// Advances like the native `operator++`, skipping keys removed meanwhile; `map.cursor_next(&mut cursor)`.
    pub fn cursor_next(&self, cursor: &mut PackedCursor) -> Option<T> {
        while let Some(block) = cursor.block {
            if let Some(key) = self.cursor_take_bit(cursor, block) {
                return Some(key);
            }
            self.cursor_next_node(cursor, block);
        }
        None
    }
    // findNext: the lowest live bit under the mask, then a mask of the bits above it.
    fn cursor_take_bit(&self, cursor: &mut PackedCursor, block: u64) -> Option<T> {
        let Some(node) = self.buckets.find(cursor.bucket, T::Word::from_bits(block)) else {
            cursor.mask = Self::FULL_MASK;
            return None;
        };
        let mut mask = T::Word::from_bits(cursor.mask);
        let key = Self::find_next(node, &mut mask);
        cursor.mask = mask.bits();
        (cursor.mask != Self::FULL_MASK).then_some(key)
    }
    // nextNode: follow the live chain link, then scan the following buckets.
    fn cursor_next_node(&self, cursor: &mut PackedCursor, block: u64) {
        let current = self.buckets.find(cursor.bucket, T::Word::from_bits(block));
        // A destroyed native cursor node would be invalid; retain Rust's safe end-of-chain extension.
        if let Some(next) = current.and_then(|node| self.buckets.following(node)) {
            cursor.block = Some(next.mask().bits() >> Self::SHIFT);
            return;
        }
        cursor.bucket += 1;
        self.cursor_find_first(cursor);
    }
    // findFirst: the head of the first non-empty bucket at or after the cursor's bucket.
    fn cursor_find_first(&self, cursor: &mut PackedCursor) {
        cursor.block = None;
        while cursor.bucket < self.buckets.len() {
            if let Some(head) = self.buckets.head(cursor.bucket) {
                cursor.block = Some(head.mask().bits() >> Self::SHIFT);
                return;
            }
            cursor.bucket += 1;
        }
    }
}
