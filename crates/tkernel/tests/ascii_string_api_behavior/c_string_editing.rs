#[test]
fn copy_c_string_and_literal_truncate_but_copy_span_keeps_nuls() {
    let mut text = AsciiString::new("old");
    text.copy_c_string(Some(b"a\0b"));
    assert_eq!(bytes(&text), b"a");
    text.copy(b"a\0b");
    assert_eq!(bytes(&text), b"a\0b");
    text.copy_literal(b"xy\0tail");
    assert_eq!(bytes(&text), b"xy");
    text.copy_c_string(None);
    assert!(text.is_empty());
}

#[test]
fn insert_c_string_and_literal_truncate_before_insertion() {
    let mut text = AsciiString::new("AZ");
    text.insert_c_string(2, Some(b"b\0c")).unwrap();
    assert_eq!(bytes(&text), b"AbZ");
    text.insert_literal(2, b"x\0ignored").unwrap();
    assert_eq!(bytes(&text), b"AxbZ");
}

#[test]
fn insert_c_string_null_bypasses_bounds_but_nonnull_empty_does_not() {
    let mut text = AsciiString::new("AZ");
    assert!(text.insert_c_string(-1, None).is_ok());
    let failure = text.insert_c_string(-1, Some(b"\0ignored")).unwrap_err();
    assert_eq!(class(failure), "Standard_OutOfRange");
    assert_eq!(bytes(&text), b"AZ");
}

#[test]
fn insert_after_c_string_and_literal_use_after_index_rules() {
    let mut text = AsciiString::new("AZ");
    text.insert_after_c_string(1, Some(b"b\0c")).unwrap();
    text.insert_after_literal(1, b"x\0ignored").unwrap();
    assert_eq!(bytes(&text), b"AxbZ");
    assert!(text.insert_after_c_string(-1, None).is_ok());
    let failure = text.insert_after_c_string(-1, Some(b"\0ignored")).unwrap_err();
    assert_eq!(class(failure), "Standard_OutOfRange");
}

#[test]
fn insert_before_c_string_and_literal_use_before_index_rules() {
    let mut text = AsciiString::new("AZ");
    text.insert_before_c_string(2, Some(b"b\0c")).unwrap();
    text.insert_before_literal(2, b"x\0ignored").unwrap();
    assert_eq!(bytes(&text), b"AxbZ");
    assert!(text.insert_before_c_string(0, None).is_ok());
    let failure = text.insert_before_c_string(0, Some(b"\0ignored")).unwrap_err();
    assert_eq!(class(failure), "Standard_OutOfRange");
}

#[test]
fn explicit_length_after_and_before_spans_keep_embedded_nuls() {
    let mut after = AsciiString::new("AZ");
    after.insert_after(1, b"b\0c").unwrap();
    assert_eq!(bytes(&after), b"Ab\0cZ");
    let mut before = AsciiString::new("AZ");
    before.insert_before(2, b"b\0c").unwrap();
    assert_eq!(bytes(&before), b"Ab\0cZ");
}
