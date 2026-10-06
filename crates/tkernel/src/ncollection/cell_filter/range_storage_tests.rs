//! Range scratch indices and stored keys share the native ten-inline coordinate policy.
use super::index_storage_tests::{IndexGeometry, inline_storage};
use super::{CellAction, CellFilter, CellIndex, indices::CellIndices};
fn range_fixture(count: usize) -> (CellFilter<IndexGeometry>, Vec<f64>, Vec<f64>) {
    let mut filter = CellFilter::with_dimension(count, 1., None);
    let minimum = vec![0.; count];
    let mut maximum = minimum.clone();
    maximum[0] = 2.;
    filter.add_range(&17, &minimum, &maximum);
    for value in 0..3 {
        let mut point = minimum.clone();
        point[0] = f64::from(value);
        filter.add(&value, &point);
    }
    (filter, minimum, maximum)
}
fn range_row(count: usize) -> String {
    let (mut filter, minimum, maximum) = range_fixture(count);
    let inline = filter.cells.items().all(|(key, _)| inline_storage(key));
    let mut row = format!("{count} {} {}", filter.cells.len(), inline as u8);
    filter
        .try_inspect_range_with(&minimum, &maximum, |target| {
            row.push_str(&format!(" {target}"));
            Ok::<_, ()>(CellAction::Purge)
        })
        .unwrap();
    row.push_str(&format!(" {}\n", filter.cells.is_empty() as u8));
    row
}
#[test]
fn native_range_add_inspect_purge_matches_inline_and_heap_rows() {
    let rows: String = [1, 3, 10, 11, 24].into_iter().map(range_row).collect();
    assert_eq!(
        rows,
        include_str!("../../../../../tools/oracle/tkernel_cell_index_ranges.expected")
    );
}
#[test]
fn range_scratch_and_yielded_indices_keep_inline_threshold_and_independent_copies() {
    for count in [1, 10, 11, 24] {
        let (filter, minimum, maximum) = range_fixture(count);
        let mut range = filter.range(&minimum, &maximum);
        let mut first = range.next().unwrap();
        assert_eq!(inline_storage(&first), count <= 10);
        first.as_mut_slice()[0] = 99;
        let rest: Vec<_> = range
            .map(|index| {
                assert_eq!(inline_storage(&index), count <= 10);
                index.as_slice()[0]
            })
            .collect();
        assert_eq!(rest, [1, 2]);
    }
}
#[test]
fn zero_axes_and_reversed_ranges_keep_existing_safe_extension() {
    let first = CellIndex::from_slice(&[2, 4]);
    let last = CellIndex::from_slice(&[1, 5]);
    assert_eq!(CellIndices::new(first.clone(), last.clone(), 2).count(), 0);
    let mut zero_axes = CellIndices::new(first.clone(), last, 0);
    assert_eq!(zero_axes.next().unwrap().as_slice(), first.as_slice());
    assert!(zero_axes.next().is_none());
}
