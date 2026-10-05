//! Supplemental source-bound checks; original legacy declaration counts stay unchanged.
use rustycad_tkernel::tcollection::{AsciiString, CatOperand, ExtendedString};

fn verify_ascii_operand<T: Clone + CatOperand + Into<AsciiString>>(
    operand: T,
    joined: &[u8],
    assigned: &[u8],
) {
    let base = AsciiString::new("h");
    assert_eq!((&base + operand.clone()).as_bytes(), joined);
    let mut destination = base.clone();
    destination += operand;
    assert_eq!(destination.as_bytes(), assigned);
    assert_eq!(base.as_bytes(), b"h");
}

fn verify_extended_operand<T: Into<ExtendedString>>(operand: T, expected: &[u16]) {
    let base = ExtendedString::from_char(104);
    assert_eq!((&base + operand).as_utf16(), expected);
    assert_eq!(base.as_utf16(), &[104]);
}

#[test]
fn ascii_nul_char_add_preserves_a_byte_that_add_assign_omits() {
    verify_ascii_operand(0_u8, b"h\0", b"h");
    verify_ascii_operand(b'x', b"hx", b"hx");
}

#[test]
fn ascii_numeric_operands_use_native_decimal_and_general_format() {
    verify_ascii_operand(-42_i32, b"h-42", b"h-42");
    verify_ascii_operand(1.25_f64, b"h1.25", b"h1.25");
}

#[test]
fn ascii_string_operand_keeps_embedded_zero_and_extended_operand_uses_utf8() {
    let mut embedded = AsciiString::default();
    embedded += b"x\0y".as_slice();
    verify_ascii_operand(&embedded, b"hx\0y", b"hx\0y");
    let extended = ExtendedString::from_char(0x03bb);
    verify_ascii_operand(&extended, "hλ".as_bytes(), "hλ".as_bytes());
}

#[test]
fn ascii_pointer_array_and_explicit_span_keep_distinct_length_contracts() {
    verify_ascii_operand("x\0y", b"hx", b"hx");
    verify_ascii_operand(b"x\0y\0", b"hx", b"hx");
    verify_ascii_operand(b"x\0y".as_slice(), b"hx\0y", b"hx\0y");
}

#[test]
fn extended_byte_and_unit_zero_operands_are_empty_strings() {
    verify_extended_operand(0_u8, &[104]);
    verify_extended_operand(b'x', &[104, 120]);
    verify_extended_operand(0_u16, &[104]);
    verify_extended_operand(0x03bb_u16, &[104, 955]);
}

#[test]
fn extended_numeric_and_string_operands_preserve_native_units() {
    verify_extended_operand(-42_i32, &[104, 45, 52, 50]);
    verify_extended_operand(1.25_f64, &[104, 49, 46, 50, 53]);
    let embedded = ExtendedString::from_raw(&[120, 0, 121]);
    verify_extended_operand(&embedded, &[104, 120, 0, 121]);
}

#[test]
fn ascii_null_c_buffer_is_a_no_op_on_original_and_concatenated_copy() {
    let mut original = AsciiString::new("h");
    assert_eq!(original.cat_c_bytes(None).as_bytes(), b"h");
    original.assign_cat_c_bytes(None);
    assert_eq!(original.as_bytes(), b"h");
}

#[test]
fn ascii_bounded_c_buffer_scans_to_first_nul_and_preserves_base_span() {
    let mut original = AsciiString::from_raw(b"h\0i");
    assert_eq!(original.cat_c_bytes(Some(b"x\0y")).as_bytes(), b"h\0ix");
    original.assign_cat_c_bytes(Some(b"x\0y"));
    assert_eq!(original.as_bytes(), b"h\0ix");
}

#[test]
fn ascii_empty_c_buffer_leaves_string_unchanged() {
    let mut original = AsciiString::new("h");
    assert_eq!(original.cat_c_bytes(Some(b"\0")).as_bytes(), b"h");
    original.assign_cat_c_bytes(Some(b"\0"));
    assert_eq!(original.as_bytes(), b"h");
}
