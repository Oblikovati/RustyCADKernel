//! Array2 inherits Array1 storage methods while preserving rectangular metadata.
use rustycad_tkernel::ncollection::{Array2, ArrayError};

#[derive(Copy, Default, Debug)]
struct RectangleCopyWithoutClone(i32);
#[allow(clippy::non_canonical_clone_impl)] // The native trivial branch bypasses copy hooks.
impl Clone for RectangleCopyWithoutClone {
    fn clone(&self) -> Self {
        panic!("RectangleCopyWithoutClone::clone must not run in native trivial copying");
    }
}
fn rectangle_copy_fixture(rows: usize, cols: usize) -> Array2<'static, RectangleCopyWithoutClone> {
    let mut array = Array2::with_size(rows, cols);
    for (offset, slot) in array.iter_mut().enumerate() {
        *slot = RectangleCopyWithoutClone(10 + offset as i32);
    }
    array
}
#[test]
fn array2_pointer_ranges_preserve_default_null_and_borrowed_empty_identity() {
    let mut array = Array2::<i32>::default();
    assert!(array.data_ptr().is_null());
    assert!(array.data_mut_ptr().is_null());
    assert!(array.data_end_ptr().is_null());
    assert!(array.data_end_mut_ptr().is_null());
    let mut buffer = [11, 22];
    let beginning = buffer.as_ptr();
    let mut borrowed = Array2::from_buffer(&mut buffer, 0, 2).unwrap();
    assert_eq!(borrowed.data_ptr(), beginning);
    assert_eq!(borrowed.data_end_mut_ptr(), beginning.cast_mut());
}
#[test]
fn array2_pointer_ranges_are_contiguous_row_major_with_arbitrary_origins() {
    let mut array = Array2::<i32>::with_bounds(-3, -2, 7, 9).unwrap();
    let beginning = array.data_ptr();
    assert_eq!(beginning, array.value(-3, 7).unwrap() as *const i32);
    assert_eq!(array.data_mut_ptr(), beginning.cast_mut());
    assert_eq!(array.data_end_ptr(), beginning.wrapping_add(6));
    assert_eq!(
        array.data_end_mut_ptr(),
        beginning.wrapping_add(6).cast_mut()
    );
}
#[test]
fn array2_trivial_assignment_reuses_equal_length_storage_and_adopts_shape() {
    let mut source = rectangle_copy_fixture(3, 2);
    source.update_lower_row(7);
    source.update_lower_col(-3);
    let mut destination = rectangle_copy_fixture(2, 3);
    let pointer = destination.data_ptr();
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(receiver, destination.assign_trivial(&source)));
    assert_eq!(destination.data_ptr(), pointer);
    assert_eq!((destination.lower_row(), destination.upper_row()), (7, 9));
    assert_eq!((destination.lower_col(), destination.upper_col()), (-3, -2));
    assert_eq!(destination.value(9, -2).unwrap().0, 15);
    assert!(std::ptr::eq(receiver, destination.assign_self()));
}
#[test]
fn array2_trivial_copy_values_checks_shape_even_when_total_lengths_match() {
    let source = rectangle_copy_fixture(3, 2);
    let mut destination = rectangle_copy_fixture(2, 3);
    let error = destination.copy_values_trivial(&source).err().unwrap();
    assert_eq!(
        error,
        ArrayError::ShapeMismatch {
            expected: (2, 3),
            actual: (3, 2)
        }
    );
    let matching = rectangle_copy_fixture(2, 3);
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(
        receiver,
        destination.copy_values_trivial(&matching).unwrap()
    ));
    let copied = destination.clone_trivial();
    assert_ne!(copied.data_ptr(), destination.data_ptr());
    assert_eq!(copied.at(1, 2).unwrap().0, 15);
}
#[test]
fn array2_owned_assignment_and_value_copy_return_the_destination() {
    let mut source = Array2::<String>::with_size(1, 2);
    source.init(String::from("source"));
    let mut destination = Array2::<String>::with_size(2, 3);
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(receiver, destination.assign(&source)));
    destination.update_lower_row(-3);
    assert!(std::ptr::eq(
        receiver,
        destination.copy_values_returning(&source).unwrap()
    ));
    assert_eq!(destination.lower_row(), -3);
    assert!(
        destination
            .copy_values_returning(&Array2::default())
            .is_err()
    );
}
#[test]
fn array2_move_assignment_transfers_borrowed_storage_and_resets_all_bounds() {
    let mut buffer = [11, 22];
    let beginning = buffer.as_ptr();
    let mut source = Array2::from_buffer(&mut buffer, 1, 2).unwrap();
    source.update_lower_row(-3);
    let mut destination = Array2::with_size(2, 3);
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(receiver, destination.move_from(&mut source)));
    assert_eq!(destination.data_ptr(), beginning);
    assert!(!destination.is_deletable());
    assert_eq!(destination.lower_row(), -3);
    assert!(source.data_ptr().is_null());
    assert_eq!((source.lower_row(), source.lower_col()), (1, 1));
    assert_eq!((source.upper_row(), source.upper_col()), (0, 0));
}
