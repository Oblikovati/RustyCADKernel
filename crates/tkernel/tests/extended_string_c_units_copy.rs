//! Null-terminated UTF-16 Copy and assignment behavior against OCCT's pointer overloads.
use rustycad_tkernel::tcollection::ExtendedString;

#[test]
fn pointer_copy_truncates_but_explicit_spans_preserve_embedded_nul() {
    let mut value = ExtendedString::new("old");
    let value_address = core::ptr::addr_of_mut!(value);
    let returned = value.copy_c_units(Some(&[65, 0, 66, 0]));
    assert_eq!(returned as *mut ExtendedString, value_address);
    assert_eq!(value.as_utf16(), &[65]);
    value.copy_span(Some(&[65, 0, 66]), 3);
    assert_eq!(value.as_utf16(), &[65, 0, 66]);
    value.copy([66, 0, 65]);
    assert_eq!(value.as_utf16(), &[66, 0, 65]);
}

#[test]
fn null_and_empty_pointer_copy_clear_the_receiver() {
    let mut value = ExtendedString::new("old");
    value.copy_c_units(None);
    assert!(value.is_empty());
    value.copy_c_units(Some(&[0, 65]));
    assert!(value.is_empty());
    value.copy_c_units(Some(&[]));
    assert!(value.is_empty());
}
