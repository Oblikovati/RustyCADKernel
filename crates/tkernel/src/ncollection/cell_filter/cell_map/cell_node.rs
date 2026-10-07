//! The native `Cell`: its index and its target list. The protected `ListNode` chain (plan D1
//! class B6) becomes an owned vector kept oldest first, so the native newest-first list order
//! is the vector reversed; targets are no longer allocated one by one from the filter arena.
use super::CellIndex;
pub(in crate::ncollection::cell_filter) struct CellRecord<T> {
    pub(in crate::ncollection::cell_filter) index: CellIndex,
    pub(super) objects: Vec<T>,
}
impl<T> CellRecord<T> {
    pub(in crate::ncollection::cell_filter) fn from_coordinates(index: &[i32]) -> Self {
        Self {
            index: CellIndex::from_slice(index),
            objects: Vec::new(),
        }
    }
    #[cfg(test)]
    pub(in crate::ncollection::cell_filter) fn len(&self) -> usize {
        self.objects.len()
    }
    /// True once every target was purged; native removes such a cell from the map.
    pub(super) fn is_vacant(&self) -> bool {
        self.objects.is_empty()
    }
}
// No API pins a target in place (the vector may move them), so a cell is Unpin whatever T is,
// as the native heap-allocated target nodes were.
impl<T> Unpin for CellRecord<T> {}

impl<T> Drop for CellRecord<T> {
    fn drop(&mut self) {
        // Native Cell::~Cell destroys its targets from the list head: newest first.
        while let Some(target) = self.objects.pop() {
            drop(target);
        }
    }
}
mod record_methods;
