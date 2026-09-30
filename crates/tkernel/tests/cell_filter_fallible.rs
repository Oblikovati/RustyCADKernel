//! Native callback failure stops traversal after preserving prior purges.
use rustycad_tkernel::ncollection::{BaseAllocator, CellAction, CellFilter, CellGeometry, List};
struct PlanarIndices;
impl CellGeometry for PlanarIndices {
    type CellTarget = i32;
    type CellPoint = [f64; 2];
    const CELL_DIMENSION: usize = 2;
    fn cell_coordinate(axis: usize, point: &Self::CellPoint) -> f64 {
        point[axis]
    }
    fn targets_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
}
#[test]
fn partial_list_failure_keeps_throwing_node_and_later_nodes() {
    let mut list = List::from_iter_in([1, 2, 3, 4], BaseAllocator::common());
    let result = list.try_retain(|value| if *value == 3 { Err(3) } else { Ok(*value == 2) });
    assert_eq!(result, Err(3));
    assert_eq!(list.iter().copied().collect::<Vec<i32>>(), [2, 3, 4]);
    list.try_retain(|_| Ok::<bool, ()>(false)).unwrap();
    assert!(list.is_empty());
}
#[test]
fn fallible_point_inspection_preserves_newest_first_early_stop() {
    let mut cells = CellFilter::<PlanarIndices>::new(1.0);
    for index in 1..=4 {
        cells.add(&index, &[0., 0.]);
    }
    let result = cells.try_inspect_with(&[0., 0.], |index| {
        if *index == 2 {
            Err(2)
        } else {
            Ok(CellAction::Purge)
        }
    });
    assert_eq!(result, Err(2));
    let remaining = collect_cells(&mut cells, [0., 0.]);
    assert_eq!(remaining, [2, 1]);
    purge_remaining_cell(&mut cells);
}
#[test]
fn fallible_range_stops_before_later_cells() {
    let mut cells = CellFilter::<PlanarIndices>::new(1.0);
    for index in 0..3 {
        cells.add(&index, &[f64::from(index), 0.]);
    }
    let result = cells.try_inspect_range_with(&[0., 0.], &[2., 0.], |index| {
        if *index == 1 {
            Err(1)
        } else {
            Ok(CellAction::Purge)
        }
    });
    assert_eq!(result, Err(1));
    let remaining = collect_cells(&mut cells, [2., 0.]);
    assert_eq!(remaining, [1, 2]);
}

fn purge_remaining_cell(cells: &mut CellFilter<PlanarIndices>) {
    cells.remove(&2, &[0., 0.]);
    cells
        .try_inspect_with(&[0., 0.], |index| {
            assert_eq!(*index, 1);
            Ok::<_, ()>(CellAction::Purge)
        })
        .unwrap();
    cells.try_inspect_with(&[0., 0.], |_| Err(())).unwrap();
}

fn collect_cells(cells: &mut CellFilter<PlanarIndices>, maximum: [f64; 2]) -> Vec<i32> {
    let mut remaining = Vec::new();
    cells
        .try_inspect_range_with(&[0., 0.], &maximum, |index| {
            remaining.push(*index);
            Ok::<_, ()>(CellAction::Keep)
        })
        .unwrap();
    remaining
}
