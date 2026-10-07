//! `Array2` has no panicking `Index` operator (plan D3): an out-of-range index is the
//! `Standard_OutOfRange` that `NCollection_Array2::Value` raises, as a named failure.
use rustycad_tkernel::ncollection::{Array2, array2_failure};

#[test]
fn out_of_range_index_is_a_named_out_of_range_failure() {
    let grid: Array2<'static, i32> = Array2::with_bounds(1, 2, 1, 3).unwrap();
    let failure = grid.value(3, 1).map_err(array2_failure).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().starts_with("NCollection_Array2: index 3"));
}

#[test]
fn in_range_access_reads_and_writes_the_cell() {
    let mut grid: Array2<'static, i32> = Array2::with_bounds(1, 2, 1, 3).unwrap();
    grid.set_value(2, 3, 9).map_err(array2_failure).unwrap();
    assert_eq!(grid.value(2, 3).copied().map_err(array2_failure).unwrap(), 9);
}

#[test]
fn invalid_bounds_are_a_range_error() {
    let Err(error) = Array2::<i32>::with_bounds(2, 0, 1, 1) else {
        panic!("bounds [2, 0] must be refused");
    };
    assert_eq!(array2_failure(error).exception_type(), "Standard_RangeError");
}
