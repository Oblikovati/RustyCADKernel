//! CellFilter construction and the native `Reset` overloads.
use super::{CellFilter, CellGeometry, CellHash};
use crate::ncollection::{Array1, BaseAllocator, DataMap};
use crate::standard::Failure;
use std::marker::PhantomData;

/// `NCollection_IncAllocator::THE_DEFAULT_BLOCK_SIZE`, used when no arena is supplied.
const DEFAULT_ARENA_BLOCK: usize = 12 * 1024;

impl<I: CellGeometry> CellFilter<I> {
    /// Use the policy dimension and a fresh arena; `CellFilter::<I>::new(1.0)`.
    pub fn new(size: f64) -> Self {
        Self::with_allocator(size, None)
    }
    /// Policy dimension, uniform `size` and an optional arena (a fresh incremental one when
    /// `None`); `CellFilter::<I>::with_allocator(1.0, Some(arena))`.
    pub fn with_allocator(size: f64, allocator: Option<BaseAllocator>) -> Self {
        Self::with_dimension(I::CELL_DIMENSION, size, allocator)
    }
    /// Explicit `dimension` overriding the policy's (native `CellFilter(theDim, ...)`);
    /// dimension 0 files every target in one cell. Example: `CellFilter::<I>::with_dimension(2, 1.0, None)`.
    pub fn with_dimension(dimension: usize, size: f64, allocator: Option<BaseAllocator>) -> Self {
        let upper: i32 = i32::try_from(dimension).expect("cell dimension fits int") - 1;
        let mut sizes: Array1<'static, f64> = Array1::with_bounds(0, upper).unwrap();
        sizes.as_mut_slice().fill(size);
        Self::assemble(dimension, sizes, allocator)
    }
    /// Anisotropic sizes, one per axis, in an injected arena; `with_sizes(vec![1.0, 2.0], arena)`.
    pub fn with_sizes(sizes: Vec<f64>, resource: BaseAllocator) -> Self {
        let upper: i32 = i32::try_from(sizes.len()).expect("cell dimension fits int") - 1;
        let mut array: Array1<'static, f64> = Array1::with_bounds(0, upper).unwrap();
        array.as_mut_slice().copy_from_slice(&sizes);
        Self::assemble(sizes.len(), array, Some(resource))
    }
    /// Native `Reset(double, alloc)`: write `size` at indices `0..dimension` of the size
    /// array, then drop all targets and adopt `allocator` (or a fresh arena).
    /// When a previous `reset_sizes` left an array whose bounds miss one of those indices,
    /// the native checked write throws Standard_OutOfRange part-way; the same error is
    /// returned here with the earlier writes kept and the targets untouched.
    /// Example: `filter.reset(2.0, None)?`.
    pub fn reset(&mut self, size: f64, allocator: Option<BaseAllocator>) -> Result<(), Failure> {
        for axis in 0..self.dimension {
            let index: i32 = axis as i32;
            *self.sizes.change_value(index).map_err(|error| {
                Failure::out_of_range(&format!("NCollection_Array1::ChangeValue: {error}"))
            })? = size;
        }
        self.reset_cells(allocator);
        Ok(())
    }
    /// Native `Reset(Array1&, alloc)`: copy the sizes (bounds included, as `Array1::Assign`),
    /// drop all targets and adopt `allocator`. The iterated dimension stays unchanged.
    /// Example: `filter.reset_sizes(&sizes, None)`.
    pub fn reset_sizes(&mut self, sizes: &Array1<'_, f64>, allocator: Option<BaseAllocator>) {
        // OCCT Array1::Assign reuses equal-length storage while rebasing its bounds.
        self.sizes.assign_trivial(sizes);
        self.reset_cells(allocator);
    }
    fn assemble(
        dimension: usize,
        sizes: Array1<'static, f64>,
        allocator: Option<BaseAllocator>,
    ) -> Self {
        let resource: BaseAllocator = arena_or_default(allocator);
        Self {
            dimension,
            sizes,
            cells: DataMap::with_allocator(CellHash, 1, resource.clone()),
            resource,
            inspector: PhantomData,
        }
    }
    fn reset_cells(&mut self, allocator: Option<BaseAllocator>) {
        self.resource = arena_or_default(allocator);
        // Native resetAllocator clears through the new arena without resetting NbBuckets.
        self.cells.clear_with_allocator(Some(self.resource.clone()));
    }
}

/// Native `resetAllocator`: a null handle means a new default incremental allocator.
fn arena_or_default(allocator: Option<BaseAllocator>) -> BaseAllocator {
    allocator.unwrap_or_else(|| BaseAllocator::incremental(DEFAULT_ARENA_BLOCK))
}

impl<I: CellGeometry> Default for CellFilter<I> {
    /// Constructs native zero-sized cells; call `filter.reset(1.0, None)` before spatial queries.
    fn default() -> Self {
        Self::new(0.0)
    }
}
