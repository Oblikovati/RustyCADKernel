//! Copy-assignment return and chaining behavior from `TCollection_ExtendedString`.
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn copy_assignment_returns_the_receiver() {
    let source = ExtendedString::from_utf16(&[65, 0x20ac, 0]);
    let mut target = ExtendedString::new("old");
    let expected_address = &mut target as *mut ExtendedString;

    let actual_address = target.copy_assign(&source) as *mut ExtendedString;

    assert_eq!(actual_address, expected_address);
    assert_eq!(target, source);
}

#[test]
fn copy_assignment_result_can_be_chained() {
    let first = ExtendedString::new("first");
    let second = ExtendedString::new("second");
    let mut target = ExtendedString::default();

    target.copy_assign(&first).copy_assign(&second);

    assert_eq!(target, second);
}

#[test]
fn copy_assignment_replaces_value_with_empty_source() {
    let empty = ExtendedString::default();
    let mut target = ExtendedString::new("not empty");
    let original_address = &mut target as *mut ExtendedString;

    let returned_address = target.copy_assign(&empty) as *mut ExtendedString;

    assert_eq!(returned_address, original_address);
    assert!(target.is_empty());
}

#[test]
fn explicit_self_assignment_is_a_noop_and_returns_receiver() {
    let mut target = ExtendedString::new("same object");
    let original = target.clone();
    let original_address = &mut target as *mut ExtendedString;

    let returned_address = target.copy_assign_self() as *mut ExtendedString;

    assert_eq!(returned_address, original_address);
    assert_eq!(target, original);
}
