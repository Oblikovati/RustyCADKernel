//! Standalone protected Cell adapters share canonical index and object ownership paths.
use rustycad_tkernel::ncollection::{
    Array1, BaseAllocator, CellFilterCell, CellFilterCellHasher, CellGeometry, HashPolicy,
};
mod cell_class_fixture;
use cell_class_fixture::{
    assert_row, compare_assignment, compare_construction, compare_self, coordinates,
};
#[test]
fn native_index_copy_and_move_construction_cover_inline_threshold() {
    for (case, size) in [0, 1, 10, 11, 24].into_iter().enumerate() {
        compare_copy(case * 3, size);
        compare_construction(case * 3 + 1, size);
    }
}
fn compare_copy(row: usize, size: usize) {
    let arena = BaseAllocator::incremental(1024);
    let mut index = coordinates(size, 10);
    let cell = CellFilterCell::<i32>::from_indices(&index, arena);
    if size > 0 {
        index[0] = 99;
    }
    assert_row(
        row,
        format!(
            "copy {size} len {} first {} distinct {} empty {}",
            cell.index().len(),
            cell.index().first().unwrap_or(&-1),
            u8::from(cell.index_data_ptr() != index.as_ptr()),
            u8::from(cell.targets().next().is_none())
        ),
    );
}
#[test]
fn native_assignment_overwrites_old_targets_and_preserves_index_storage_policy() {
    for (case, (from, to)) in [(3, 3), (3, 11), (11, 3), (11, 11)].into_iter().enumerate() {
        compare_assignment(15 + case * 2, from, to);
    }
}
#[test]
fn native_self_assignment_discards_head_without_target_destruction() {
    compare_self(23, 3);
    compare_self(25, 11);
}
struct Geometry;
impl CellGeometry for Geometry {
    type CellTarget = i32;
    type CellPoint = [f64; 3];
    const CELL_DIMENSION: usize = 3;
    fn cell_coordinate(axis: usize, point: &Self::CellPoint) -> f64 {
        point[axis]
    }
    fn targets_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
}
#[test]
fn native_point_constructor_equality_and_hasher_ignore_object_bookkeeping() {
    let arena = BaseAllocator::incremental(1024);
    let mut sizes = Array1::with_bounds(-2, 0).unwrap();
    sizes.as_mut_slice().copy_from_slice(&[2., 0.5, 4.]);
    let point = CellFilterCell::from_point::<Geometry>(&[5., -1.9, 12.], &sizes, arena.clone());
    let mut equal = CellFilterCell::from_indices(&[2, -3, 3], arena.clone());
    equality_row(&point, &equal);
    equal.prepend_target(&7);
    assert_row(
        28,
        format!(
            "objects-ignored {} {}",
            u8::from(point.is_equal(&equal)),
            u8::from(point.hash_code() == equal.hash_code())
        ),
    );
}
fn equality_row(point: &CellFilterCell<i32>, equal: &CellFilterCell<i32>) {
    let arena = BaseAllocator::incremental(1024);
    let shorter = CellFilterCell::from_indices(&[2, 3], arena.clone());
    let unequal = CellFilterCell::from_indices(&[2, 3, 4], arena);
    let hasher = CellFilterCellHasher;
    assert_eq!(point.index(), [2, -3, 3]);
    assert_row(
        27,
        format!(
            "point 2 -3 3 hash {} equal {} {} {} {} {}",
            hasher.hash(point),
            u8::from(point.is_equal(equal)),
            u8::from(point == equal),
            u8::from(hasher.equal(point, equal)),
            u8::from(point.is_equal(&shorter)),
            u8::from(point.is_equal(&unequal))
        ),
    );
}
#[test]
fn native_discard_retains_incremental_bytes_while_rust_releases_reset_leases() {
    let arena = BaseAllocator::incremental(1024);
    let before = arena.allocate_raw(8).unwrap();
    let mut cell = CellFilterCell::from_indices(&[0], arena.clone());
    cell.prepend_target(&7);
    let object = std::ptr::from_ref(cell.targets().next().unwrap()).addr();
    cell.move_self_discard();
    let after = arena.allocate_raw(8).unwrap();
    let delta = after.as_ptr().addr() - before.as_ptr().addr();
    let node_bytes = after.as_ptr().addr() - object;
    assert_row(29, format!("retained delta {delta} node {node_bytes}"));
    drop(cell);
    assert!(arena.reset(false).is_ok());
}
