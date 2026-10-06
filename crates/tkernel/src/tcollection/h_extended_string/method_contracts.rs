//! Native raw buffer and converting constructor expressions.
use super::{ExtendedString, HExtendedString};
use crate::standard::{Failure, exceptions};
impl HExtendedString {
    /// Returns native ToExtString identity; later edits through any handle can invalidate it; `text.to_ext_ptr()`.
    pub fn to_ext_ptr(&self) -> *const u16 {
        self.string().to_ext_ptr()
    }
    /// Transfers the wrapped native string allocation and resets its source; `HExtendedString::from_moved_string(&mut text)`.
    pub fn from_moved_string(source: &mut ExtendedString) -> Self {
        ExtendedString::move_from(source).into()
    }
    /// Checks the native nullable byte-string constructor; `HExtendedString::from_optional_c_string(Some(b"abc"))?`.
    pub fn from_optional_c_string(source: Option<&[u8]>) -> Result<Self, Failure> {
        source
            .map(Self::from_c_string)
            .ok_or_else(|| null_constructor("byte"))
    }
    /// Checks the native nullable UTF-16 constructor; `HExtendedString::from_optional_utf16(Some(&[65, 0]))?`.
    pub fn from_optional_utf16(source: Option<&[u16]>) -> Result<Self, Failure> {
        source
            .map(Self::from_utf16)
            .ok_or_else(|| null_constructor("UTF-16"))
    }
}
fn null_constructor(encoding: &str) -> Failure {
    exceptions::null_object(&format!(
        "TCollection_HExtendedString constructor: null {encoding} pointer; expected a terminated string"
    ))
}
