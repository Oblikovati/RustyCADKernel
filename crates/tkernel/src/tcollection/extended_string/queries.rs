//! Element access, classification and the numeric conversions.
use super::{
    ExtendedString,
    errors::{int_length, offset, out_of_range},
};
use crate::{
    standard::{Failure, exceptions},
    tcollection::AsciiString,
};
impl ExtendedString {
    /// Returns the signed length in code units (`Length`); `text.length() == 5`.
    pub fn length(&self) -> i32 {
        int_length(self.len())
    }
    /// Reads the code unit at a one-based position in 1..=len; `text.value(2)?`.
    pub fn value(&self, index: i32) -> Result<u16, Failure> {
        if index < 1 || index > self.length() {
            return Err(out_of_range(
                "TCollection_ExtendedString::Value : parameter theWhere",
                format_args!("theWhere {index}; expected 1..={}", self.len()),
            ));
        }
        Ok(self.terminated[offset(index)])
    }
    /// True when every unit is in OCCT's eight-bit "ASCII range" (`IsAnAscii`: no bit of
    /// 0xFF00 set); `ExtendedString::from_char(255).is_ascii()`.
    pub fn is_ascii(&self) -> bool {
        self.as_utf16().iter().all(|&unit| unit & 0xFF00 == 0)
    }
    /// Parses the UTF-8 form with `strtol`; zero for an empty string, Standard_NumericError
    /// when no digit is read; `text.integer_value()?`.
    pub fn integer_value(&self) -> Result<i32, Failure> {
        if self.is_empty() {
            return Ok(0);
        }
        self.as_ascii().integer_value().map_err(|_| {
            self.numeric_error("TCollection_AsciiString::IntegerValue", "a decimal integer")
        })
    }
    /// Whether the UTF-8 form starts with an integer not followed by a '.'; false when
    /// empty; `text.is_integer_value()`.
    pub fn is_integer_value(&self) -> bool {
        !self.is_empty() && self.as_ascii().is_integer_value()
    }
    /// Parses the UTF-8 form with OCCT's `Strtod`; zero for an empty string,
    /// Standard_NumericError when nothing is read; `text.real_value()?`.
    pub fn real_value(&self) -> Result<f64, Failure> {
        if self.is_empty() {
            return Ok(0.0);
        }
        self.as_ascii()
            .real_value()
            .map_err(|_| self.numeric_error("TCollection_AsciiString::RealValue", "a real number"))
    }
    /// Whether the UTF-8 form starts with (or, when `full`, entirely is) a real value;
    /// false when empty; `text.is_real_value(true)`.
    pub fn is_real_value(&self, full: bool) -> bool {
        !self.is_empty() && self.as_ascii().is_real_value(full)
    }
    /// OCCT converts with `TCollection_AsciiString(*this)`: UTF-8, no replacement.
    fn as_ascii(&self) -> AsciiString {
        AsciiString::from_extended(self, None)
    }
    /// The Standard_NumericError that TCollection_AsciiString raises for this content.
    fn numeric_error(&self, occt_message: &str, expected: &str) -> Failure {
        exceptions::numeric_error(&format!(
            "{occt_message} (text {:?}; expected {expected})",
            self.to_utf8()
        ))
    }
}
