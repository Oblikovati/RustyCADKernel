#[test]
fn c_string_search_truncates_at_nul_while_span_search_does_not() {
    let text = AsciiString::from_raw(b"a\0b--a");
    assert_eq!(text.search_c_string(Some(b"b\0a")), 3);
    assert_eq!(text.search(b"b\0a"), -1);
    assert_eq!(text.search_from_end_c_string(Some(b"a\0b")), 6);
    assert_eq!(text.search_from_end(b"a\0b"), 1);
}

#[test]
fn c_string_search_matches_null_empty_and_leading_nul_rules() {
    let text = AsciiString::new("abcabc");
    assert_eq!(text.search_c_string(None), -1);
    assert_eq!(text.search_from_end_c_string(None), -1);
    assert_eq!(text.search_c_string(Some(b"")), -1);
    assert_eq!(text.search_from_end_c_string(Some(b"\0abc")), -1);
    assert_eq!(text.search_c_string(Some(b"ca\0abc")), 3);
    assert_eq!(text.search_from_end_c_string(Some(b"ca\0abc")), 3);
}

#[test]
fn set_literal_uses_array_extent_and_preserves_interior_nuls() {
    let text = AsciiString::from_raw(b"x\0y");
    assert_eq!(text.first_location_in_set_literal(b"z\0x\0", 1, 3).unwrap(), 1);
    assert_eq!(text.first_location_in_set(b"z\0x\0", 1, 3).unwrap(), 1);
    assert_eq!(text.first_location_not_in_set_literal(b"x\0", 1, 3).unwrap(), 2);
    assert_eq!(text.first_location_not_in_set(b"x\0", 1, 3).unwrap(), 3);

    let with_nul = AsciiString::from_raw(b"a\0b");
    assert_eq!(with_nul.first_location_in_set_literal(b"x\0z\0", 1, 3).unwrap(), 2);
    assert_eq!(with_nul.first_location_not_in_set_literal(b"a\0z\0", 1, 3).unwrap(), 3);
}

#[test]
fn empty_literal_set_short_circuits_before_range_validation() {
    let text = AsciiString::new("x");
    assert_eq!(text.first_location_in_set_literal(b"\0", 0, 9).unwrap(), 0);
    assert_eq!(text.first_location_not_in_set_literal(b"\0", 0, 9).unwrap(), 0);
    assert_eq!(text.first_location_in_set([], 0, -1).unwrap(), 0);
    assert_eq!(text.first_location_not_in_set([], 0, -1).unwrap(), 0);
}
