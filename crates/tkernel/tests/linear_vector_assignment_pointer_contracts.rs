//! Native linear-vector pointer identity and Copy assignment without Clone hooks.
use rustycad_tkernel::ncollection::LinearVector;

#[test]
fn linear_pointer_endpoints_distinguish_null_reserved_and_initialized_storage() {
    let mut vector = LinearVector::<i32>::default();
    assert!(vector.data_ptr().is_null());
    assert!(vector.data_mut_ptr().is_null());
    assert!(vector.data_end_ptr().is_null());
    assert!(vector.data_end_mut_ptr().is_null());
    vector.reserve(8);
    let beginning = vector.data_ptr();
    assert!(!beginning.is_null());
    assert_eq!(vector.data_end_ptr(), beginning);
    vector.append(11);
    vector.append(22);
    assert_eq!(vector.data_mut_ptr(), beginning.cast_mut());
    assert_eq!(vector.data_end_ptr(), beginning.wrapping_add(2));
    assert_eq!(
        vector.data_end_mut_ptr(),
        beginning.wrapping_add(2).cast_mut()
    );
}

#[test]
fn linear_clear_retains_or_releases_pointer_storage_as_requested() {
    let mut vector = LinearVector::with_value(3, 42);
    let pointer = vector.data_ptr();
    vector.clear(false);
    assert_eq!(vector.data_ptr(), pointer);
    assert_eq!(vector.data_end_ptr(), pointer);
    vector.clear(true);
    assert!(vector.data_ptr().is_null());
}

#[derive(Copy, Default)]
struct LinearAssignmentCopyOnly(i32);
// Native memcpy must never dispatch the user-defined copying hook.
#[allow(clippy::non_canonical_clone_impl)]
impl Clone for LinearAssignmentCopyOnly {
    fn clone(&self) -> Self {
        panic!("LinearAssignmentCopyOnly::clone must not run during native memcpy");
    }
}
fn linear_copy_fixture(size: usize, value: i32) -> LinearVector<LinearAssignmentCopyOnly> {
    let mut vector = LinearVector::with_capacity(size);
    for _ in 0..size {
        vector.append(LinearAssignmentCopyOnly(value));
    }
    vector
}
#[test]
fn linear_trivial_assignment_reuses_capacity_without_clone_hooks() {
    let mut destination = linear_copy_fixture(8, 11);
    let source = linear_copy_fixture(3, 22);
    let pointer = destination.data_ptr();
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(receiver, destination.assign_trivial(&source)));
    assert_eq!(destination.data_ptr(), pointer);
    assert_eq!(destination.capacity(), 8);
    assert_eq!(destination.len(), 3);
    assert!(destination.iter().all(|item| item.0 == 22));
    destination.assign_trivial(&LinearVector::default());
    assert_eq!(destination.data_ptr(), pointer);
    assert!(destination.is_empty());
}
#[test]
fn linear_trivial_assignment_grows_to_source_length_without_clone_hooks() {
    let mut destination = linear_copy_fixture(1, 11);
    let source = linear_copy_fixture(7, 22);
    destination.assign_trivial(&source);
    assert_eq!(destination.capacity(), 7);
    assert_eq!(destination.len(), 7);
    assert!(destination.iter().all(|item| item.0 == 22));
}
#[test]
fn linear_owned_copy_and_move_assignment_return_destination() {
    let mut destination = LinearVector::with_value(2, String::from("old"));
    let mut source = LinearVector::with_value(7, String::from("source"));
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(receiver, destination.assign(&source)));
    assert_eq!(destination.as_slice(), source.as_slice());
    let pointer = source.data_ptr();
    assert!(std::ptr::eq(receiver, destination.move_from(&mut source)));
    assert_eq!(destination.data_ptr(), pointer);
    assert!(source.data_ptr().is_null());
    assert!(std::ptr::eq(receiver, destination.assign_self()));
    assert_eq!(destination.data_ptr(), pointer);
}
#[test]
fn linear_owned_copy_assignment_returns_receiver_when_reusing_capacity() {
    let mut destination = LinearVector::with_value(8, String::from("old"));
    let source = LinearVector::with_value(3, String::from("source"));
    let pointer = destination.data_ptr();
    let receiver = &destination as *const _;
    assert!(std::ptr::eq(receiver, destination.assign(&source)));
    assert_eq!(destination.data_ptr(), pointer);
    assert_eq!(destination.as_slice(), source.as_slice());
}

#[test]
fn linear_trivial_copy_constructor_uses_length_without_clone_hooks() {
    let mut source = linear_copy_fixture(8, 22);
    for _ in 0..5 {
        source.erase_last();
    }
    let copied = source.clone_trivial();
    assert_eq!(copied.capacity(), 3);
    assert_eq!(copied.len(), 3);
    assert!(copied.iter().all(|item| item.0 == 22));
    assert_ne!(copied.data_ptr(), source.data_ptr());
    assert_eq!(source.capacity(), 8);
    assert!(
        LinearVector::<i32>::default()
            .clone_trivial()
            .data_ptr()
            .is_null()
    );
}
