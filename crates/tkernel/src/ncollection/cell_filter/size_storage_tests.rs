//! Preserve the equal-length storage reuse inherited from native Array1::Assign.
use super::{CellFilter, CellGeometry};
use crate::ncollection::Array1;

struct StorageGeometry;
type SizeProbe = CellFilter<StorageGeometry>;
impl CellGeometry for StorageGeometry {
    type CellTarget = i32;
    type CellPoint = [f64; 1];
    const CELL_DIMENSION: usize = 1;
    fn cell_coordinate(axis: usize, point: &[f64; 1]) -> f64 {
        point[axis]
    }
    fn targets_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
}
fn cell_size_fixture_array(lower: i32, values: &[f64]) -> Array1<'static, f64> {
    let mut array = Array1::with_bounds(lower, lower + values.len() as i32 - 1).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}
fn cell_size_reset_row(filter: &mut SizeProbe, name: &str, source: &Array1<'_, f64>) -> String {
    let before = filter.sizes.as_slice().as_ptr();
    let same_len = filter.sizes.len() == source.len();
    filter.reset_sizes(source, None);
    let retained = !same_len || before == filter.sizes.as_slice().as_ptr();
    format!(
        "{name} {} {} {} {} {} {}",
        u8::from(retained),
        filter.sizes.lower(),
        filter.sizes.upper(),
        filter.dimension,
        filter.sizes.as_slice()[0],
        u8::from(filter.cells.is_empty())
    )
}
fn initial_size_rows(filter: &mut SizeProbe) -> Vec<String> {
    filter.add(&7, &[0.0]);
    let mut source = cell_size_fixture_array(4, &[2.0]);
    let mut rows = vec![cell_size_reset_row(filter, "rebase", &source)];
    source.as_mut_slice()[0] = 9.0;
    rows.push(format!(
        "independent {} {}",
        filter.sizes.as_slice()[0],
        source.as_slice()[0]
    ));
    rows
}
#[test]
fn native_size_reset_reuses_equal_length_storage_and_rebases_bounds() {
    let mut filter = SizeProbe::new(1.0);
    let mut rows = initial_size_rows(&mut filter);
    rows.push(cell_size_reset_row(
        &mut filter,
        "grow",
        &cell_size_fixture_array(-3, &[4.0, 5.0]),
    ));
    rows.push(cell_size_reset_row(
        &mut filter,
        "reuse_two",
        &cell_size_fixture_array(8, &[6.0, 7.0]),
    ));
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../../../../tools/oracle/tkernel_cell_size_storage.expected")
    );
}
#[test]
fn repeated_same_length_resets_keep_one_live_size_allocation() {
    let mut filter = CellFilter::<StorageGeometry>::new(1.0);
    let address = filter.sizes.as_slice().as_ptr();
    for lower in -4..=4 {
        filter.reset_sizes(&cell_size_fixture_array(lower, &[f64::from(lower)]), None);
        assert_eq!(filter.sizes.as_slice().as_ptr(), address);
        assert_eq!(filter.sizes.lower(), lower);
        assert_eq!(filter.sizes.as_slice(), &[f64::from(lower)]);
    }
}
