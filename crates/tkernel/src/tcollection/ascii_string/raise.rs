//! The exceptions AsciiString members raise, with the offending value in the message.
//!
//! The exception class is OCCT's; the message starts with the member name OCCT reports
//! and adds the offending value and the accepted shape, which OCCT often leaves empty.
use crate::standard::{Failure, exceptions};

/// `Standard_OutOfRange` from `member`; `out_of_range("Value", format!("index 0; expected 1..=3"))`.
pub(crate) fn out_of_range(member: &str, detail: String) -> Failure {
    exceptions::out_of_range(&format!("TCollection_AsciiString::{member}: {detail}"))
}

/// `Standard_NegativeValue` for a negative width; `negative_width("Center", -1)`.
pub(crate) fn negative_width(member: &str, width: i32) -> Failure {
    exceptions::negative_value(&format!(
        "TCollection_AsciiString::{member}: width {width}; expected >= 0"
    ))
}

/// `Standard_NumericError` for text without a number; `no_number("IntegerValue", b"x")`.
pub(crate) fn no_number(member: &str, text: &[u8]) -> Failure {
    exceptions::numeric_error(&format!(
        "TCollection_AsciiString::{member}: text {:?}; expected a leading number",
        String::from_utf8_lossy(text)
    ))
}

/// Checks OCCT's `from > 0 && to <= length && from <= to` range, naming `member` on failure.
pub(crate) fn check_range(member: &str, from: i32, to: i32, length: i32) -> Result<(), Failure> {
    if from > 0 && to <= length && from <= to {
        return Ok(());
    }
    Err(out_of_range(
        member,
        format!("range {from}..={to}; expected 1 <= from <= to <= {length}"),
    ))
}
