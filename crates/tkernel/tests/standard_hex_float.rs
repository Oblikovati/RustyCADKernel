//! Supplemental C-locale oracle rows, not additional legacy test declarations.
use rustycad_tkernel::standard::c_string::{FormatArg, sprintf_into_slice};

const HEX_ROWS: &str = include_str!("../../../tools/oracle/tkernel_hex_float.expected");

#[test]
fn hexadecimal_double_formats_match_pinned_native_rows() {
    let mut count = 0;
    for row in HEX_ROWS.lines() {
        assert_hex_row(row);
        count += 1;
    }
    assert_eq!(count, 1312);
}

fn assert_hex_row(row: &str) {
    let fields: Vec<&str> = row.split('|').collect();
    assert_eq!(fields.len(), 4);
    let value = f64::from_bits(u64::from_str_radix(fields[0], 16).unwrap());
    let expected_count: usize = fields[2].parse().unwrap();
    let expected: Vec<u8> = decode_hex(fields[3]);
    let mut actual = [0xa5_u8; 512];
    let count =
        sprintf_into_slice(&mut actual, fields[1].as_bytes(), &[FormatArg::Real(value)]).unwrap();
    assert_eq!(count, expected_count, "{row}");
    assert_eq!(&actual[..count], expected, "{row}");
    assert_eq!(actual[count], 0);
    assert!(actual[count + 1..].iter().all(|&byte| byte == 0xa5));
}

fn decode_hex(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 2, 0);
    (0..encoded.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&encoded[index..index + 2], 16).unwrap())
        .collect()
}

#[test]
fn hexadecimal_formats_validate_types_and_dynamic_dimensions() {
    let mut output = [0xa5_u8; 64];
    let arguments = [FormatArg::Int(14), FormatArg::Int(1), FormatArg::Real(1.5)];
    let count = sprintf_into_slice(&mut output, b"%+0*.*A", &arguments).unwrap();
    assert_eq!(&output[..count], b"+0X000001.8P+0");
    let before = output;
    assert!(sprintf_into_slice(&mut output, b"%a", &[FormatArg::Int(1)]).is_err());
    assert!(sprintf_into_slice(&mut output, b"%lla", &[FormatArg::Real(1.0)]).is_err());
    assert_eq!(output, before);
}
