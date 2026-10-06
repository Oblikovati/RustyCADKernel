//! Array1 raw storage, returning assignment and trivial copying contracts.
use rustycad_tkernel::ncollection::{Array1, ArrayError};

#[test]
fn array1_pointer_ranges_preserve_empty_owned_and_borrowed_identity() {
    let mut empty = Array1::<i32>::default();
    assert!(empty.data_ptr().is_null());
    assert!(empty.data_mut_ptr().is_null());
    assert!(empty.data_end_ptr().is_null());
    assert!(empty.data_end_mut_ptr().is_null());
    let mut buffer = [11, 22, 33];
    let beginning = buffer.as_ptr();
    let mut view = Array1::from_slice(&mut buffer[..0]);
    assert_eq!(view.data_ptr(), beginning);
    assert_eq!(view.data_mut_ptr(), beginning.cast_mut());
    assert_eq!(view.data_end_ptr(), beginning);
    assert_eq!(view.data_end_mut_ptr(), beginning.cast_mut());
}
#[test]
fn array1_pointer_ranges_use_storage_order_without_lower_bound_bias() {
    let mut array = Array1::<i32>::with_bounds(-3, -1).unwrap();
    let beginning = array.data_ptr();
    assert_eq!(beginning, array.first().unwrap() as *const i32);
    assert_eq!(array.data_mut_ptr(), beginning.cast_mut());
    assert_eq!(array.data_end_ptr(), beginning.wrapping_add(3));
    assert_eq!(
        array.data_end_mut_ptr(),
        beginning.wrapping_add(3).cast_mut()
    );
    array.update_lower_bound(100);
    assert_eq!(array.data_ptr(), beginning);
}
#[test]
fn array1_shared_mutable_pointer_detaches_using_existing_safe_view_policy() {
    let buffer = [11, 22];
    let mut view = Array1::from_shared(&buffer, 4);
    assert_eq!(view.data_ptr(), buffer.as_ptr());
    assert_ne!(view.data_mut_ptr(), buffer.as_ptr().cast_mut());
    assert!(view.is_deletable());
    view[4] = 99;
    assert_eq!(buffer, [11, 22]);
}
#[derive(Copy, Default, Debug)]
struct ArrayCopyWithoutClone(i32);
// The native trivial branch must not invoke a user-defined copying hook.
#[allow(clippy::non_canonical_clone_impl)]
impl Clone for ArrayCopyWithoutClone {
    fn clone(&self) -> Self {
        panic!("ArrayCopyWithoutClone::clone must not run in the native trivial branch");
    }
}
fn array1_copy_fixture(lower: i32, length: usize) -> Array1<'static, ArrayCopyWithoutClone> {
    let mut array = Array1::with_size(length);
    array.update_lower_bound(lower);
    for (offset, value) in array.iter_mut().enumerate() {
        *value = ArrayCopyWithoutClone(10 + offset as i32);
    }
    array
}
#[test]
fn array1_trivial_assignment_reuses_same_size_storage_and_rebases() {
    let source = array1_copy_fixture(7, 3);
    let mut destination = array1_copy_fixture(-2, 3);
    let pointer = destination.data_ptr();
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(receiver, destination.assign_trivial(&source)));
    assert_eq!(destination.data_ptr(), pointer);
    assert_eq!((destination.lower(), destination.upper()), (7, 9));
    assert_eq!(destination[8].0, 11);
    assert!(std::ptr::eq(receiver, destination.assign_self()));
    assert_eq!(destination.data_ptr(), pointer);
}
#[test]
fn array1_trivial_assignment_detaches_different_size_and_copies_without_clone() {
    let source = array1_copy_fixture(7, 3);
    let mut buffer = [ArrayCopyWithoutClone(99); 2];
    let mut destination = Array1::from_slice(&mut buffer);
    destination.assign_trivial(&source);
    assert!(destination.is_deletable());
    assert_eq!(destination[9].0, 12);
    let copied = destination.clone_trivial();
    assert_eq!(copied.lower(), 7);
    assert_ne!(copied.data_ptr(), destination.data_ptr());
    assert_eq!(copied[8].0, 11);
    assert_eq!(buffer[0].0, 99);
}
#[test]
fn array1_trivial_copy_values_preserves_bounds_and_rejects_length_mismatch() {
    let source = array1_copy_fixture(7, 3);
    let mut destination = array1_copy_fixture(-2, 3);
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(
        receiver,
        destination.copy_values_trivial(&source).unwrap()
    ));
    assert_eq!((destination.lower(), destination.upper()), (-2, 0));
    let error = destination
        .copy_values_trivial(&array1_copy_fixture(0, 2))
        .unwrap_err();
    assert_eq!(
        error,
        ArrayError::DimensionMismatch {
            expected: 3,
            actual: 2
        }
    );
    assert_eq!(destination[-1].0, 11);
}
#[test]
fn array1_owned_assignment_and_copy_values_return_the_receiver() {
    let buffer = [String::from("source")];
    let source = Array1::from_shared(&buffer, 7);
    let mut destination = Array1::<String>::with_size(2);
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(receiver, destination.assign(&source)));
    destination.update_lower_bound(-2);
    assert!(std::ptr::eq(
        receiver,
        destination.copy_values_returning(&source).unwrap()
    ));
    assert_eq!(destination.lower(), -2);
    assert_eq!(destination[-2], "source");
    assert!(
        destination
            .copy_values_returning(&Array1::default())
            .is_err()
    );
}
#[test]
fn array1_move_assignment_returns_receiver_and_transfers_borrowed_storage() {
    let mut buffer = [11, 22];
    let pointer = buffer.as_ptr();
    let mut source = Array1::from_slice(&mut buffer);
    source.update_lower_bound(-3);
    let mut destination = Array1::with_size(3);
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(receiver, destination.move_from(&mut source)));
    assert_eq!(destination.data_ptr(), pointer);
    assert!(!destination.is_deletable());
    assert_eq!(destination.lower(), -3);
    assert!(source.data_ptr().is_null());
    assert_eq!((source.lower(), source.upper()), (1, 0));
}
