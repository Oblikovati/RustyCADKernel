#[test]
fn assign_cat_char_ignores_nul_but_cat_char_appends_it() {
    let mut text = AsciiString::new("ab");
    text.assign_cat(0_u8);
    text += 0_u8;
    assert_eq!(bytes(&text), b"ab");
    assert_eq!(bytes(&text.cat(0_u8)), b"ab\0");
    assert_eq!(bytes(&(&text + 0_u8)), b"ab\0");
    assert_eq!(bytes(&text.cat_char(b'!')), b"ab!");
}

#[test]
fn cat_operands_follow_their_overloads() {
    let text = AsciiString::new("x");
    assert_eq!(bytes(&text.cat("a\0b")), b"xa");
    assert_eq!(bytes(&text.cat(&b"a\0b"[..])), b"xa\0b");
    assert_eq!(bytes(&text.cat(b"yz")), b"xyz");
    assert_eq!(bytes(&(&text + -3)), b"x-3");
    assert_eq!(bytes(&(&text + 0.5)), b"x0.5");
    assert_eq!(bytes(&text.cat_wide(&[0x20AC, 0])), "x€".as_bytes());
}

#[test]
fn prepend_and_self_concatenation() {
    let mut text = AsciiString::new("tail");
    text.prepend(&AsciiString::from_raw(b"h\0"));
    assert_eq!(bytes(&text), b"h\0tail");
    text.assign_cat_self(2..4);
    assert_eq!(bytes(&text), b"h\0tailta");
}

#[test]
fn insert_char_reports_too_big_before_too_small() {
    let mut text = AsciiString::new("ac");
    text.insert_char(2, b'b').unwrap();
    text.insert_char(4, 0).unwrap();
    assert_eq!(bytes(&text), b"abc\0");
    assert_eq!(class(text.insert_char(6, b'x').unwrap_err()), "Standard_OutOfRange");
    assert_eq!(class(text.insert_char(0, b'x').unwrap_err()), "Standard_OutOfRange");
}

#[test]
fn insert_after_and_before_check_their_own_ranges() {
    let mut text = AsciiString::new("ac");
    text.insert_after(1, "b").unwrap();
    text.insert_after(0, ">").unwrap();
    text.insert_before(4, "<").unwrap();
    assert_eq!(bytes(&text), b">ab<c");
    assert!(text.insert_after(-1, "x").is_err());
    assert!(text.insert_before(6, "x").is_err());
    assert!(text.insert(7, "x").unwrap_err().is_kind("Standard_RangeError"));
}

#[test]
fn set_value_char_refuses_nul_and_the_end_position() {
    let mut text = AsciiString::new("abc");
    text.set_value_char(3, b'C').unwrap();
    assert_eq!(bytes(&text), b"abC");
    assert!(text.set_value_char(4, b'x').is_err());
    assert!(text.set_value_char(1, 0).is_err());
}

#[test]
fn set_value_grows_but_refuses_gaps() {
    let mut text = AsciiString::new("abc");
    text.set_value(3, "XYZ").unwrap();
    assert_eq!(bytes(&text), b"abXYZ");
    text.set_value(6, "!").unwrap();
    assert_eq!(bytes(&text), b"abXYZ!");
    assert!(text.set_value(8, "?").is_err());
    assert!(text.set_value(0, "?").is_err());
}

#[test]
fn remove_checks_the_end_and_refuses_undefined_inputs() {
    let mut text = AsciiString::new("abcdef");
    text.remove(2, 3).unwrap();
    assert_eq!(bytes(&text), b"aef");
    text.remove(4, 0).unwrap();
    assert!(text.remove(3, 2).is_err());
    assert!(text.remove(0, 1).is_err());
    assert!(text.remove(1, -1).is_err());
    assert!(text.remove(i32::MAX, i32::MAX).is_err());
}

#[test]
fn remove_all_with_case_folds_ascii_only() {
    let mut text = AsciiString::from_raw(b"aAbB\xe9\xc9");
    text.remove_all_with_case(b'a', false);
    assert_eq!(bytes(&text), b"bB\xe9\xc9");
    text.remove_all_with_case(0xE9, false);
    assert_eq!(bytes(&text), b"bB\xc9");
}

#[test]
fn truncate_and_split_bounds() {
    let mut text = AsciiString::from_raw(b"ab\0cd");
    let tail = text.split(1).unwrap();
    assert_eq!((bytes(&text), bytes(&tail)), (&b"a"[..], &b"b"[..]));
    assert!(text.split(2).is_err());
    assert!(text.truncate(-1).is_err());
    text.truncate(0).unwrap();
    assert!(text.is_empty());
}

#[test]
fn copy_self_from_the_start_is_a_no_op() {
    let mut text = AsciiString::new("abcdef");
    text.copy_self(0..2);
    assert_eq!(bytes(&text), b"abcdef");
    text.copy_self(4..6);
    assert_eq!(bytes(&text), b"ef");
}

#[test]
fn justification_and_centering() {
    let mut text = AsciiString::new("ab");
    text.center(5, b'*').unwrap();
    assert_eq!(bytes(&text), b"**ab*");
    text.left_justify(6, b'.').unwrap();
    text.right_justify(7, b'>').unwrap();
    assert_eq!(bytes(&text), b">**ab*.");
    text.center(3, b'!').unwrap();
    assert_eq!(bytes(&text), b">**ab*.");
    assert_eq!(class(text.center(-1, b'!').unwrap_err()), "Standard_NegativeValue");
    assert!(text.left_justify(i32::MIN, b'!').is_err());
}

#[test]
fn adjust_and_change_all_use_the_c_locale() {
    let mut text = AsciiString::from_raw(b"\t\x0b a\xa0 \r\n");
    text.left_adjust();
    text.right_adjust();
    assert_eq!(bytes(&text), b"a\xa0");
    let mut other = AsciiString::new("aAbA");
    other.change_all(b'a', b'x', false);
    assert_eq!(bytes(&other), b"xxbx");
    other.change_all(b'X', b'y', true);
    assert_eq!(bytes(&other), b"xxbx");
}
