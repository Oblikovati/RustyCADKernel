//! Null-terminated UTF-16 comparison and affix adapters against OCCT's pointer overloads.
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn predicates_stop_at_nul_while_explicit_spans_keep_all_units() {
    let one = ExtendedString::from_utf16(&[65, 0]);
    let longer = ExtendedString::from_utf16(&[65, 66, 0]);
    let mixed = [65, 0, 66, 0];
    assert!(one.is_equal_c_units(Some(&mixed)));
    assert!(!one.is_different_c_units(Some(&mixed)));
    assert!(!one.is_less_c_units(Some(&mixed)));
    assert!(!one.is_greater_c_units(Some(&mixed)));
    assert!(!one.is_equal_span(Some(&mixed[..3]), 3));
    assert!(one.is_less_span(Some(&mixed[..3]), 3));
    assert!(longer.is_different_c_units(Some(&mixed)));
    assert!(!longer.is_less_c_units(Some(&mixed)));
    assert!(longer.is_greater_c_units(Some(&mixed)));
    assert!(longer.starts_with_c_units(Some(&mixed)));
    assert!(!longer.ends_with_c_units(Some(&mixed)));
    assert!(!longer.starts_with_span(Some(&mixed[..3]), 3));
    assert!(!longer.ends_with_span(Some(&mixed[..3]), 3));
    assert!(longer.ends_with_c_units(Some(&[66, 0, 65])));
}

#[test]
fn null_pointer_predicates_use_native_zero_length_rules() {
    let empty = ExtendedString::default();
    let nonempty = ExtendedString::from_utf16(&[65, 0]);
    assert!(empty.is_equal_c_units(None));
    assert!(!empty.is_different_c_units(None));
    assert!(!empty.is_less_c_units(None));
    assert!(!empty.is_greater_c_units(None));
    assert!(empty.starts_with_c_units(None));
    assert!(empty.ends_with_c_units(None));
    assert!(!nonempty.is_equal_c_units(None));
    assert!(nonempty.is_different_c_units(None));
    assert!(!nonempty.is_less_c_units(None));
    assert!(nonempty.is_greater_c_units(None));
    assert!(nonempty.starts_with_c_units(None));
    assert!(nonempty.ends_with_c_units(None));
}
