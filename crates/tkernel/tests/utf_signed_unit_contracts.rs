//! Native signed-byte dispatch preserves bits even for malformed legacy encodings.
use rustycad_tkernel::ncollection::{UtfIterator, UtfString, UtfUnit};

fn native_row(row: &str) {
    let expected = include_str!("../../../tools/oracle/tkernel_utf_signed_units.expected");
    assert!(
        expected.lines().any(|line| line == row),
        "expected native row, got {row}"
    );
}
fn signed_input_row(label: &str, bytes: &[i8], count: usize) -> String {
    let mut iter = UtfIterator::new(bytes);
    let mut row = label.to_owned();
    for _ in 0..count {
        row.push_str(&format!(
            " {}:{}:{}:{}",
            iter.current(),
            iter.index(),
            iter.buffer_here(),
            iter.buffer_next()
        ));
        iter.advance();
    }
    row
}
fn signed_output_row(code: u32) -> String {
    let source = [code, 0];
    let iter = UtfIterator::new(&source);
    let mut output = [17_i8; 5];
    let mut overload = output;
    let count = iter.get_utf(&mut output).unwrap();
    assert_eq!(iter.get_utf8_signed(&mut overload), Ok(count));
    assert_eq!(output, overload);
    let mut row = format!("write {code} {} {count}", iter.advance_bytes_utf::<i8>());
    for unit in output {
        row.push_str(&format!(" {unit}"));
    }
    row
}
#[test]
fn signed_units_decode_native_positions() {
    native_row(&signed_input_row(
        "input",
        &[65, -61, -87, -16, -97, -104, -128, 0, 0],
        4,
    ));
}
#[test]
fn signed_units_preserve_native_malformed_sequences() {
    native_row(&signed_input_row(
        "invalid",
        &[-128, -64, -128, -4, -128, -128, -128, -128, -128, 0, 0],
        4,
    ));
}
#[test]
fn signed_outputs_match_native_boundaries_and_untouched_tail() {
    for code in [
        0, 65, 127, 128, 2047, 2048, 0xd800, 0xffff, 0x10000, 0x1f600, 0x10ffff, 0x110000,
    ] {
        native_row(&signed_output_row(code));
    }
}
#[test]
fn signed_output_reports_short_buffer_without_partial_write() {
    let iter = UtfIterator::new(&[0x1f600_u32]);
    let mut output = [17_i8; 3];
    let error = iter.get_utf8_signed(&mut output).unwrap_err();
    assert_eq!((error.needed, error.available), (4, 3));
    assert_eq!(output, [17; 3]);
}
#[test]
fn unit_bit_conversions_preserve_each_storage_width() {
    assert_eq!(u8::from_bits(0x1ff).to_bits(), 255);
    assert_eq!(u16::from_bits(0x1ffff).to_bits(), 65535);
    assert_eq!(u32::from_bits(u32::MAX).to_bits(), u32::MAX);
    for bits in 0..=255 {
        assert_eq!(i8::from_bits(bits).to_bits(), bits);
    }
}
#[test]
fn signed_utf_strings_copy_and_transcode_without_sign_extension() {
    let source = [-61_i8, -87, 0];
    let copied = UtfString::<i8>::from_unicode_units(&source, -1);
    let unsigned = UtfString::<u8>::from_unicode_units(&source, -1);
    let wide = UtfString::<u16>::from_unicode_units(&source, -1);
    assert_eq!(copied.to_c_string(), &source);
    assert_eq!(unsigned.to_c_string(), &[0xc3, 0xa9, 0]);
    assert_eq!(wide.to_c_string(), &[0xe9, 0]);
}
#[test]
fn signed_postincrement_and_remaining_output_preserve_positions() {
    let mut iter = UtfIterator::new(&[-61_i8, -87, 65, 0]);
    assert_eq!(iter.post_increment().current(), 233);
    assert_eq!((iter.current(), iter.buffer_here()), (65, 2));
    let mut output = [17_i8; 3];
    assert_eq!(iter.get_utf_remaining(&mut output).unwrap(), &[17, 17]);
    assert_eq!(output, [65, 17, 17]);
}

#[test]
fn signed_lead_byte_matrix_matches_native_decoding() {
    for lead in 0..=255 {
        let padded = [lead as i8, -128, -128, -128, -128, -128, 0, 0];
        native_row(&signed_input_row(&format!("lead{lead}"), &padded, 1));
    }
}
