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

#[test]
fn prepend_self_scans_suffix_and_preserves_embedded_zero() {
    let mut value = ExtendedString::new("ABC");
    value.prepend_c_units_from_self(1).unwrap();
    assert_eq!(value.as_utf16(), &[66, 67, 65, 66, 67]);

    let mut embedded = ExtendedString::from_raw(&[65, 66, 0, 67, 68]);
    embedded.prepend_c_units_from_self(3).unwrap();
    assert_eq!(embedded.as_utf16(), &[67, 68, 65, 66, 0, 67, 68]);
}

#[test]
fn prepend_self_from_zero_copies_full_text_and_end_is_empty() {
    let mut value = ExtendedString::new("ABCDEFGH");
    value.prepend_c_units_from_self(0).unwrap();
    assert_eq!(
        value.as_utf16(),
        "ABCDEFGHABCDEFGH".encode_utf16().collect::<Vec<_>>()
    );
    let unchanged = value.clone();
    let end = value.len();
    value.prepend_c_units_from_self(end).unwrap();
    assert_eq!(value, unchanged);
}

#[test]
fn prepend_self_rejects_offsets_beyond_the_terminator() {
    let mut value = ExtendedString::new("ABC");
    let error = value.prepend_c_units_from_self(4).unwrap_err();
    assert!(error.to_string().contains("start 4"));
    assert!(error.to_string().contains("expected 0..=3"));
    assert_eq!(value.as_utf16(), &[65, 66, 67]);
}

#[test]
fn set_value_self_preserves_native_forward_overlap_order() {
    let mut target_after_source = embedded_self_source();
    target_after_source
        .set_value_c_units_from_self(2, 0)
        .unwrap();
    assert_eq!(target_after_source.as_utf16(), &[97, 97, 97, 97, 97, 101]);

    let mut target_before_source = embedded_self_source();
    target_before_source
        .set_value_c_units_from_self(1, 2)
        .unwrap();
    assert_eq!(target_before_source.as_utf16(), &[99, 100, 99, 100, 0, 101]);
}

#[test]
fn set_value_self_accepts_terminator_and_rejects_invalid_positions() {
    let mut value = embedded_self_source();
    let unchanged = value.clone();
    value.set_value_c_units_from_self(7, 6).unwrap();
    assert_eq!(value, unchanged);

    let failure = value.set_value_c_units_from_self(0, 6).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
    assert!(failure.message().contains("theWhere 0"));
}

#[test]
fn set_value_self_rejects_native_reallocation_alias_domain() {
    let mut value = embedded_self_source();
    let failure = value.set_value_c_units_from_self(4, 0).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NotImplemented");
    assert!(failure.message().contains("end <= Length() 6"));
    assert_eq!(value.as_utf16(), &[97, 98, 99, 100, 0, 101]);
}

fn embedded_self_source() -> ExtendedString {
    let mut value = ExtendedString::default();
    value.copy([97, 98, 99, 100, 0, 101]);
    value
}
