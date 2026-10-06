//! Search the native bitmap directly; no secondary key map or copied object list.
use super::TypedSparseStorage;
use crate::ncollection::{SparseArrayBase, SparseArrayBlock};
impl<T> TypedSparseStorage<T> {
    pub(in super::super) fn first_key(&self, start: usize) -> Option<usize> {
        first_raw_sparse_key(&self.base, start)
    }
    pub(in super::super) fn has_allocated_blocks(&self) -> bool {
        (0..self.base.block_slots()).any(|number| !self.base.block_address(number).is_null())
    }
}
pub(in super::super) fn first_raw_sparse_key(
    base: &SparseArrayBase,
    start: usize,
) -> Option<usize> {
    if start >= base.capacity() {
        return None;
    }
    for number in start / base.block_size()..base.block_slots() {
        if let Some(key) = first_raw_sparse_block_key(base, number, start) {
            return Some(key);
        }
    }
    None
}
fn first_raw_sparse_block_key(
    base: &SparseArrayBase,
    number: usize,
    start: usize,
) -> Option<usize> {
    let address = base.block_address(number);
    if address.is_null() {
        return None;
    }
    let block = unsafe { base.get_block(address) };
    let beginning = number * base.block_size();
    first_sparse_bit(block, start.saturating_sub(beginning), base.block_size())
        .map(|offset| beginning + offset)
}
fn first_sparse_bit(block: SparseArrayBlock, start: usize, size: usize) -> Option<usize> {
    (start..size).find(|&offset| unsafe { block.is_set(offset) != 0 })
}
