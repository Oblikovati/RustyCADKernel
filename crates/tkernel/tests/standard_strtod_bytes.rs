//! Native C-string byte-domain regressions for the Strtod and Atof wrappers.
use rustycad_tkernel::standard::{c_string::atof_bytes, strtod_bytes};

const NATIVE_CASES: &str = include_str!("../../../tools/oracle/strtod_bytes.expected");

#[test]
fn standard_strtod_bytes_matches_pinned_native_fixture() {
    for row in NATIVE_CASES.lines().filter(|line| !line.starts_with('#')) {
        check_native_row(row);
    }
}

#[test]
fn standard_atof_bytes_uses_native_prefix_and_nul_rules() {
    assert_eq!(atof_bytes(b" 1.25\xffsuffix"), 1.25);
    assert_eq!(atof_bytes(b"42\0\xffsuffix"), 42.0);
}

fn check_native_row(row: &str) {
    let mut fields = row.split_ascii_whitespace();
    let input = decode_hex(fields.next().expect("fixture input"));
    let expected_bits = u64::from_str_radix(fields.next().expect("fixture bits"), 16).unwrap();
    let expected_end = fields
        .next()
        .expect("fixture end offset")
        .parse::<usize>()
        .unwrap();
    let (value, end) = strtod_bytes(&input);
    assert_eq!(
        (value.to_bits(), end),
        (expected_bits, expected_end),
        "{row}"
    );
}

fn decode_hex(hex: &str) -> Vec<u8> {
    let (pairs, remainder) = hex.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty(), "odd hex fixture: {hex}");
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
