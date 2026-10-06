//! Protected state observed by native derived-class oracles.
use super::{CellFilter, CellGeometry};
use crate::ncollection::{Array1, BaseAllocator};
struct LineGeometry;
impl CellGeometry for LineGeometry {
    type CellTarget = i32;
    type CellPoint = [f64; 1];
    const CELL_DIMENSION: usize = 1;
    fn cell_coordinate(axis: usize, point: &Self::CellPoint) -> f64 {
        point[axis]
    }
    fn targets_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
}
fn populated_filter(arena: BaseAllocator) -> CellFilter<LineGeometry> {
    let mut filter = CellFilter::with_allocator(1.0, Some(arena));
    for index in 0..103 {
        filter.add(&index, &[f64::from(index)]);
    }
    assert_eq!(filter.cells.bucket_count(), 1009);
    filter
}
#[test]
fn reset_retains_bucket_modulus_with_same_and_replacement_allocator() {
    let arena = BaseAllocator::incremental(1024);
    let mut filter = populated_filter(arena.clone());
    filter.reset(2.0, Some(arena.clone())).unwrap();
    assert_eq!(filter.cells.bucket_count(), 1009);
    assert!(filter.cells.is_empty());
    assert_eq!(filter.cells.allocator(), &arena);
    filter.reset(3.0, None).unwrap();
    assert_eq!(filter.cells.bucket_count(), 1009);
    assert_ne!(filter.cells.allocator(), &arena);
    assert_eq!(filter.cells.allocator(), &filter.resource);
}
#[test]
fn reset_size_array_keeps_dimension_bucket_count_and_copies_array_bounds() {
    let arena = BaseAllocator::incremental(1024);
    let mut filter = populated_filter(arena.clone());
    let mut sizes = Array1::with_bounds(4, 5).unwrap();
    sizes.as_mut_slice().copy_from_slice(&[2.0, 3.0]);
    filter.reset_sizes(&sizes, Some(arena));
    assert_eq!(filter.dimension, 1);
    assert_eq!(filter.cells.bucket_count(), 1009);
    assert_eq!((filter.sizes.lower(), filter.sizes.upper()), (4, 5));
    assert_eq!(filter.sizes.as_slice(), &[2.0, 3.0]);
}
#[test]
fn default_filter_keeps_native_zero_size_until_initialized() {
    let mut filter = CellFilter::<LineGeometry>::default();
    assert_eq!(filter.dimension, 1);
    assert_eq!(filter.cells.bucket_count(), 1);
    assert_eq!(filter.sizes.as_slice(), &[0.0]);
    filter.reset(1.0, None).unwrap();
    assert_eq!(filter.sizes.as_slice(), &[1.0]);
}
