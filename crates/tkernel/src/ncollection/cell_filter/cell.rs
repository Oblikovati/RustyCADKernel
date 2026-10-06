//! Standalone native Cell operations use the same record and object nodes as the map engine.
#![allow(unsafe_code)]
use super::{CellGeometry, cell_map::cell_node::CellRecord};
use crate::ncollection::destructor_unwind::DeferredDestructorPanic;
use crate::ncollection::{Array1, BaseAllocator, HashPolicy};
/// Owning adapter for protected native `Cell`, with allocator leases outside its native fields.
/// `CellFilterCell::<i32>::from_indices(&[1, 2], arena)` constructs a lookup-only cell.
pub struct CellFilterCell<T> {
    record: CellRecord<T>,
    allocator: BaseAllocator,
}
impl<T> CellFilterCell<T> {
    /// Copy coordinates into an empty cell; `CellFilterCell::<i32>::from_indices(&[1], arena)`.
    pub fn from_indices(index: &[i32], allocator: BaseAllocator) -> Self {
        Self {
            record: CellRecord::from_coordinates(index),
            allocator,
        }
    }
    /// Apply native coordinate conversion and size-array length; `CellFilterCell::from_point::<I>(&point, &sizes, arena)`.
    pub fn from_point<I: CellGeometry<CellTarget = T>>(
        point: &I::CellPoint,
        sizes: &Array1<'_, f64>,
        allocator: BaseAllocator,
    ) -> Self {
        Self {
            record: CellRecord::from_point::<I>(point, sizes),
            allocator,
        }
    }
    /// Borrow the cell key; `assert_eq!(cell.index(), &[1, 2])`.
    pub fn index(&self) -> &[i32] {
        self.record.index.as_slice()
    }
    /// Observe retained empty index allocations; `cell.index_data_ptr()` remains valid while borrowed.
    pub fn index_data_ptr(&self) -> *const i32 {
        self.record.index.data_ptr()
    }
    /// Compare only coordinates, regardless of targets; `cell.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.record.is_equal(&other.record)
    }
    /// Hash the native coordinate bytes; `cell.hash_code()` ignores its object head.
    pub fn hash_code(&self) -> usize {
        self.record.hash_code()
    }
    /// Borrow newest-first targets; `cell.targets().count()` follows actual native links.
    pub fn targets(&self) -> impl Iterator<Item = &T> {
        self.record.targets()
    }
    /// Transfer index storage and object ownership, leaving the source empty; `CellFilterCell::move_construct(&mut source)`.
    pub fn move_construct(source: &mut Self) -> Self {
        Self {
            record: CellRecord::move_construct(&mut source.record),
            allocator: source.allocator.clone(),
        }
    }
    /// Native assignment discards existing targets without invoking their destructors.
    /// `destination.move_from_discard(&mut source)` can leak resources owned by discarded targets.
    /// Rust releases their storage leases; incremental block bytes remain retained. Other allocator
    /// kinds reclaim node storage, an extension to native incremental-only object allocation.
    pub fn move_from_discard(&mut self, source: &mut Self) -> &mut Self {
        self.record
            .move_from_discard(&mut source.record, &self.allocator);
        self.allocator = source.allocator.clone();
        self
    }
    /// Native self assignment preserves the index but discards its object head without Target Drop.
    /// `cell.move_self_discard()` can leak target-owned resources; storage release follows `move_from_discard`.
    pub fn move_self_discard(&mut self) -> &mut Self {
        self.record.discard_objects(&self.allocator);
        self
    }
}
impl<T: Clone> CellFilterCell<T> {
    /// Build a native Object-first node in this cell's arena; `cell.prepend_target(&value)`.
    /// This safely adapts external initialization of protected `Cell::Objects`.
    pub fn prepend_target(&mut self, target: &T) {
        self.record.prepend(target, &self.allocator);
    }
}
impl<T> PartialEq for CellFilterCell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl<T> Eq for CellFilterCell<T> {}
impl<T> Drop for CellFilterCell<T> {
    fn drop(&mut self) {
        let mut panics = DeferredDestructorPanic::default();
        unsafe {
            CellRecord::drop_objects(
                std::ptr::addr_of_mut!(self.record),
                &self.allocator,
                &mut panics,
            )
        };
        panics.finish(Ok(()));
    }
}
/// Native `CellHasher` callable adapters; `CellFilterCellHasher.hash(&cell)` hashes its index.
#[derive(Clone, Copy, Default)]
pub struct CellFilterCellHasher;
impl<T> HashPolicy<CellFilterCell<T>> for CellFilterCellHasher {
    fn hash(&self, cell: &CellFilterCell<T>) -> usize {
        cell.hash_code()
    }
    fn equal(&self, first: &CellFilterCell<T>, second: &CellFilterCell<T>) -> bool {
        first.is_equal(second)
    }
}
