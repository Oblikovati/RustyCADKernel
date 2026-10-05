//! Null-terminated UTF-16 mutation behavior compared with OCCT pointer overloads.
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn bounded_pointer_mutations_stop_at_nul_and_spans_keep_embedded_nul() {
    let source = [65, 0, 66];
    assert_eq!(
        ExtendedString::new("XY")
            .cat_c_units(Some(&source))
            .as_utf16(),
        &[88, 89, 65]
    );
    assert_eq!(inserted_pointer().as_utf16(), &[88, 65, 89]);
    assert_eq!(set_pointer().as_utf16(), &[88, 65]);
    assert_eq!(prepended_pointer().as_utf16(), &[65, 88, 89]);
    assert_eq!(inserted_span().as_utf16(), &[88, 65, 0, 66, 89]);
}

fn inserted_pointer() -> ExtendedString {
    let mut value = ExtendedString::new("XY");
    value.insert_c_units(2, Some(&[65, 0, 66])).unwrap();
    value
}

fn set_pointer() -> ExtendedString {
    let mut value = ExtendedString::new("XY");
    value.set_value_c_units(2, Some(&[65, 0, 66])).unwrap();
    value
}

fn prepended_pointer() -> ExtendedString {
    let mut value = ExtendedString::new("XY");
    value.prepend_c_units(Some(&[65, 0, 66]));
    value
}

fn inserted_span() -> ExtendedString {
    let mut value = ExtendedString::new("XY");
    value.insert_span(2, Some(&[65, 0, 66]), 3).unwrap();
    value
}

#[test]
fn null_empty_and_nul_first_inputs_keep_native_noop_behavior() {
    let mut inserted = ExtendedString::new("XY");
    inserted.insert_c_units(2, None).unwrap();
    inserted.insert_c_units(2, Some(&[])).unwrap();
    inserted.insert_c_units(2, Some(&[0, 65])).unwrap();
    assert_eq!(inserted.as_utf16(), &[88, 89]);

    let mut replaced = inserted.clone();
    replaced.set_value_c_units(2, None).unwrap();
    replaced.set_value_c_units(2, Some(&[])).unwrap();
    replaced.set_value_c_units(2, Some(&[0, 65])).unwrap();
    assert_eq!(replaced.as_utf16(), &[88, 89]);
    replaced.prepend_c_units(None);
    replaced.prepend_c_units(Some(&[0, 65]));
    assert_eq!(replaced.as_utf16(), &[88, 89]);
}

#[test]
fn insert_and_set_validate_position_before_null_input() {
    let mut value = ExtendedString::new("XY");
    let insert_error = value.insert_c_units(0, None).unwrap_err();
    let set_error = value.set_value_c_units(0, None).unwrap_err();
    assert!(insert_error.to_string().contains("theWhere 0"));
    assert!(set_error.to_string().contains("theWhere 0"));
    assert_eq!(value.as_utf16(), &[88, 89]);
}
