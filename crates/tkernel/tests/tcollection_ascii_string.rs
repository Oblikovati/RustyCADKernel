//! Original AsciiString declarations retain byte-level checks and one-based offsets.
#![allow(non_snake_case, clippy::approx_constant, useless_ptr_null_checks)]
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString};
fn text(actual: &AsciiString, expected: &str) {
    assert_eq!(actual.as_bytes(), expected.as_bytes());
}
include!("tcollection_ascii_string/basic.rs");
include!("tcollection_ascii_string/concatenation.rs");
include!("tcollection_ascii_string/insertion.rs");
include!("tcollection_ascii_string/mutation.rs");
include!("tcollection_ascii_string/comparison.rs");
include!("tcollection_ascii_string/views.rs");
include!("tcollection_ascii_string/regressions.rs");
include!("tcollection_ascii_string/useful_length.rs");
