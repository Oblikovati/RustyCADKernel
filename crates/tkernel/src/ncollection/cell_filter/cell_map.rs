//! The filter's native `NCollection_Map<Cell>` on the safe slot storage of `Map`.
use super::CellIndex;
use crate::ncollection::{BaseAllocator, BaseMap, HashPolicy, chained_slots::ChainedSlots};
pub(super) mod cell_node;
mod retention;
use cell_node::CellRecord;
pub(super) struct CellMap<T> {
    storage: ChainedSlots<CellRecord<T>, ()>,
}
#[derive(Clone, Copy)]
struct CellMapHash;
impl<T> HashPolicy<CellRecord<T>> for CellMapHash {
    fn hash(&self, cell: &CellRecord<T>) -> usize {
        cell.hash_code()
    }
    fn equal(&self, first: &CellRecord<T>, second: &CellRecord<T>) -> bool {
        first.is_equal(second)
    }
}
impl<T> CellMap<T> {
    pub(super) fn new(allocator: BaseAllocator) -> Self {
        Self {
            storage: ChainedSlots::new(1, allocator),
        }
    }
    /// Native `myCells.Clear(myAllocator)`: drops every cell; a new allocator releases the table.
    pub(super) fn clear_with_allocator(&mut self, requested: Option<BaseAllocator>) {
        self.storage.clear_with_allocator(requested);
    }
    /// Native `Contained(Cell(index))`: the slot of the cell with `index`, if any.
    fn lookup_index(&self, index: &[i32]) -> Option<usize> {
        let probe = CellRecord::from_coordinates(index);
        self.storage
            .lookup(&probe, &CellMapHash)
            .map(|(_, slot)| slot)
    }
    // Native TryEmplaced: grow first, construct the lookup Cell, then hash it once.
    fn prepare_cell(&mut self, index: &[i32]) -> &mut CellRecord<T> {
        let size = self.storage.size();
        if size == 0 || size > self.storage.nb_buckets() {
            self.storage.resize(size, &CellMapHash);
        }
        let cell = CellRecord::from_coordinates(index);
        let (bucket, existing) = self.storage.lookup_with_bucket(&cell, &CellMapHash);
        let slot = existing.unwrap_or_else(|| self.storage.insert(bucket, || cell, || ()));
        self.storage.payload_mut().key_mut(slot)
    }
    // Native Map::Remove of a cell whose target list became empty.
    fn remove_slot(&mut self, slot: usize) {
        drop(self.storage.remove(slot));
    }
}
impl<T: Clone> CellMap<T> {
    pub(super) fn add(&mut self, index: &CellIndex, target: &T) {
        self.add_index(index.as_slice(), target);
    }
    pub(super) fn add_index(&mut self, index: &[i32], target: &T) {
        self.prepare_cell(index).prepend(target);
    }
}
#[cfg(test)]
impl<T> CellMap<T> {
    pub(super) fn len(&self) -> usize {
        self.storage.size()
    }
    pub(super) fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub(super) fn bucket_count(&self) -> usize {
        self.storage.nb_buckets()
    }
    pub(super) fn allocator(&self) -> &BaseAllocator {
        self.storage.allocator()
    }
    /// Every cell in native bucket order.
    pub(super) fn items(&self) -> impl Iterator<Item = (&CellIndex, &CellRecord<T>)> {
        crate::ncollection::slot_chains::SlotRange::new(self.storage.view())
            .map(|(cell, ())| (&cell.index, cell))
    }
}

#[cfg(test)]
mod native_tests;
