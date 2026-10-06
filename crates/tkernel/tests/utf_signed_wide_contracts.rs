//! Preserve OCCT signed promotion even when negative UTF-16 units resemble surrogate bits.
use rustycad_tkernel::ncollection::{UtfIterator, UtfString, UtfUnit};

const SIGNED_WORD_CASES: [i32; 13] = [
    i32::MIN,
    -1,
    0,
    127,
    128,
    2047,
    2048,
    0xd800,
    0xffff,
    0x10000,
    0x10ffff,
    0x110000,
    i32::MAX,
];

fn assert_wide_native(row: &str) {
    let expected = include_str!("../../../tools/oracle/tkernel_utf_signed_wide.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "expected native row, got {row}"
    );
}
fn wide_read_row<T: UtfUnit>(label: &str, source: &[T], count: usize) -> String {
    let mut cursor = UtfIterator::new(source);
    let mut row = label.to_owned();
    for _ in 0..count {
        row.push_str(&format!(
            " {}:{}:{}:{}",
            cursor.current(),
            u8::from(cursor.is_valid()),
            cursor.buffer_next(),
            cursor.advance_bytes_utf8()
        ));
        cursor.advance();
    }
    row
}
fn wide_encoding_row<T: UtfUnit + Into<i64>>(label: &str, value: T) -> String {
    let source = [value, T::default(), T::default()];
    let cursor = UtfIterator::new(&source);
    let output = wide_output_text(&cursor);
    format!(
        "{label} {} {} {}{output}",
        value.into(),
        cursor.current(),
        u8::from(cursor.is_valid())
    )
}
fn wide_output_text<T: UtfUnit>(cursor: &UtfIterator<'_, T>) -> String {
    let (mut bytes, mut halves, mut word) = ([17_u8; 4], [17_u16; 2], [17_u32; 1]);
    let mut row = format!(
        " {} {} {}",
        cursor.get_utf8(&mut bytes).unwrap(),
        cursor.get_utf16(&mut halves).unwrap(),
        cursor.get_utf32(&mut word).unwrap()
    );
    append_wide_units(&mut row, bytes, halves, word);
    row
}
fn append_wide_units(row: &mut String, bytes: [u8; 4], halves: [u16; 2], word: [u32; 1]) {
    for unit in bytes
        .map(u32::from)
        .into_iter()
        .chain(halves.map(u32::from))
        .chain(word)
    {
        row.push_str(&format!(" {unit}"));
    }
}
#[test]
fn signed_wide_decoding_preserves_native_promotion_and_positions() {
    assert_wide_native(&wide_read_row("i16", &[65_i16, -10179, -8704, -1, 0, 0], 5));
    assert_wide_native(&wide_read_row(
        "u16",
        &[65_u16, 0xd83d, 0xde00, 0xffff, 0, 0],
        4,
    ));
    assert_wide_native(&wide_read_row("i32", &[65_i32, 0x1f600, -1, 0, 0], 4));
}
#[test]
fn signed_short_sources_match_native_unsigned_output_boundaries() {
    for value in [
        -32768_i16, -32767, -10179, -8704, -1, 0, 1, 127, 128, 255, 256, 2047, 2048, 32767,
    ] {
        assert_wide_native(&wide_encoding_row("encode16", value));
    }
}
#[test]
fn signed_word_sources_match_native_unsigned_output_boundaries() {
    for value in SIGNED_WORD_CASES {
        assert_wide_native(&wide_encoding_row("encode32", value));
    }
}
#[test]
fn signed_wide_storage_bits_remain_distinct_from_promoted_symbols() {
    assert_eq!(i16::from_bits(0xffff).to_bits(), 65535);
    assert_eq!(UtfIterator::new(&[-1_i16]).current(), u32::MAX);
    assert_eq!(i32::from_bits(u32::MAX).to_bits(), u32::MAX);
    assert_eq!(UtfIterator::new(&[-1_i32]).current(), u32::MAX);
}
#[test]
fn signed_output_adapter_preserves_unsigned_encoding_bits() {
    for code in [
        0_u32,
        0x41,
        0x80,
        0xd800,
        0xffff,
        0x1f600,
        0x110000,
        u32::MAX,
    ] {
        assert_signed_encoding_bits(code);
    }
}

fn assert_signed_encoding_bits(code: u32) {
    let source = [code];
    let cursor = UtfIterator::new(&source);
    let (mut signed16, mut unsigned16) = ([17_i16; 2], [17_u16; 2]);
    assert_eq!(
        cursor.get_utf(&mut signed16),
        cursor.get_utf16(&mut unsigned16)
    );
    assert_eq!(signed16.map(|unit| unit as u16), unsigned16);
    let mut signed32 = [17_i32];
    assert_eq!(cursor.get_utf(&mut signed32), Ok(1));
    assert_eq!(signed32[0] as u32, code);
}
#[test]
fn signed_strings_retain_storage_bits_and_drop_invalid_conversions() {
    let source = [-10179_i16, -8704, 65, 0];
    let copied = UtfString::<i16>::from_unicode_units(&source, -1);
    let converted = UtfString::<u8>::from_unicode_units(&source, -1);
    assert_eq!(copied.to_c_string(), &source);
    assert_eq!(converted.to_c_string(), b"A\0");
    assert_eq!(converted.length(), 3);
    let wide = UtfString::<i32>::from_unicode_units(&[0x1f600_u32, 0], -1);
    assert_eq!(wide.to_c_string(), &[0x1f600, 0]);
}
