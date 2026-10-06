//! Native Cell constructors/hash oracle, exercised through the filter's actual key storage.
use super::{CellFilter, CellGeometry, CellHash, CellIndex};
use crate::ncollection::{Array1, HashPolicy};
pub(super) struct IndexGeometry;
impl CellGeometry for IndexGeometry {
    type CellTarget = i32;
    type CellPoint = Vec<f64>;
    const CELL_DIMENSION: usize = 3;
    fn cell_coordinate(axis: usize, point: &Self::CellPoint) -> f64 {
        point[axis]
    }
    fn targets_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
}
pub(super) fn inline_storage(index: &CellIndex) -> bool {
    let begin = std::ptr::from_ref(index).addr();
    let buffer = index.as_slice().as_ptr().addr();
    (begin..begin + std::mem::size_of::<CellIndex>()).contains(&buffer)
}
fn index_from_native_point(count: usize) -> CellIndex {
    let point: Vec<f64> = (0..count)
        .map(|i| (i as f64 - 4.75) * (i as f64 + 1.))
        .collect();
    let mut sizes = Array1::with_bounds(-7, count as i32 - 8).unwrap();
    for (i, size) in sizes.as_mut_slice().iter_mut().enumerate() {
        *size = i as f64 + 1.;
    }
    CellIndex::from_point::<IndexGeometry>(&point, &sizes)
}
fn native_row(count: usize) -> String {
    let original = index_from_native_point(count);
    let copied = CellIndex::from_slice(original.as_slice());
    let mut row = row_header(count, &original, &copied);
    for value in original.as_slice() {
        row.push_str(&format!(" {value}"));
    }
    let mut changed = copied.as_slice().to_vec();
    changed[0] += 1;
    let copied = CellIndex::from_slice(&changed);
    row.push_str(&format!(" {}\n", CellHash.equal(&original, &copied) as u8));
    row
}
fn row_header(count: usize, original: &CellIndex, copied: &CellIndex) -> String {
    let independent = original.as_slice().as_ptr() != copied.as_slice().as_ptr();
    format!(
        "{count} {} {} {} {} {}",
        inline_storage(original) as u8,
        inline_storage(copied) as u8,
        independent as u8,
        CellHash.equal(original, copied) as u8,
        CellHash.hash(original)
    )
}
#[test]
#[cfg(target_pointer_width = "64")]
fn cell_index_storage_and_hash_match_five_native_rows() {
    let rows: String = [1, 3, 10, 11, 24].into_iter().map(native_row).collect();
    assert_eq!(
        rows,
        include_str!("../../../../../tools/oracle/tkernel_cell_index_storage.expected")
    );
}
#[test]
fn cell_index_equality_checks_dimension_and_lookup_copy_is_independent() {
    let first = CellIndex::from_slice(&[1, 2]);
    let second = CellIndex::from_slice(&[1, 2, 0]);
    assert!(!CellHash.equal(&first, &second));
    let mut source = vec![1, 2];
    let copied = CellIndex::from_slice(&source);
    source[0] = 99;
    assert!(CellHash.equal(&first, &copied));
    assert_eq!(format!("{copied:?}"), "[1, 2]");
}
#[test]
fn actual_filter_keys_use_native_inline_threshold() {
    for count in [1, 3, 10, 11, 24] {
        let mut filter = CellFilter::<IndexGeometry>::with_dimension(count, 1., None);
        let point = vec![2.; count];
        filter.add(&7, &point);
        let (key, targets) = filter.cells.items().next().unwrap();
        assert_eq!(inline_storage(key), count <= 10);
        assert_eq!(key.as_slice(), vec![2; count]);
        assert_eq!(targets.len(), 1);
        filter.remove(&7, &point);
        assert!(filter.cells.is_empty());
    }
}
