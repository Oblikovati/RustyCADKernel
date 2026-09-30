//! UTF-16 legacy assertions, including aliasing and invalid pointer/length adapters.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::{
    ncollection::DataMap,
    tcollection::{AsciiString, ExtendedString},
};
fn text(string: &ExtendedString, expected: &str) {
    assert_eq!(
        AsciiString::from_extended(string, None).as_bytes(),
        expected.as_bytes()
    );
}
fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn contains(string: &ExtendedString, pattern: &str) -> bool {
    AsciiString::from_extended(string, None)
        .as_bytes()
        .windows(pattern.len())
        .any(|part| part == pattern.as_bytes())
}
include!("tcollection_extended_string/basic.rs");
include!("tcollection_extended_string/conversion.rs");
include!("tcollection_extended_string/adjustment.rs");
include!("tcollection_extended_string/queries.rs");
include!("tcollection_extended_string/boundaries.rs");
include!("tcollection_extended_string/aliasing.rs");
