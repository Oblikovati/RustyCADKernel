//! `AssignCat`, `Cat`, `Prepend` and their operators.
use super::AsciiString;
use crate::tcollection::ExtendedString;

impl AsciiString {
    /// `AssignCat`/`operator+=` for every operand with an `AsciiString` conversion: a `&str`
    /// is a C string (through its first NUL), a byte slice is an explicit span, `u8` is a
    /// `char` (NUL appends nothing), `i32` is `%d` and `f64` is `%g`; `text.assign_cat(42)`.
    pub fn assign_cat(&mut self, input: impl Into<Self>) {
        self.append(&input.into());
    }
    /// `Cat`/`operator+`: a concatenated copy. Unlike `assign_cat`, a `u8` operand is
    /// appended even when it is NUL, as `Cat(char)` copies one byte; `text.cat("suffix")`.
    pub fn cat(&self, input: impl CatOperand) -> Self {
        input.cat_onto(self)
    }
    /// `Cat(char)`/`operator+(char)`: appends exactly one byte, NUL included; `text.cat_char(b'!')`.
    pub fn cat_char(&self, byte: u8) -> Self {
        Self::joined_with_char(self, byte)
    }
    /// `AssignCat(const TCollection_ExtendedString&, char)`: appends UTF-8, or one byte per
    /// UTF-16 unit with `replacement` for units above 0xFF; `text.assign_cat_extended(&wide, None)`.
    pub fn assign_cat_extended(&mut self, text: &ExtendedString, replacement: Option<u8>) {
        self.append(&Self::from_extended(text, replacement));
    }
    /// `Cat(const TCollection_ExtendedString&, char)`; `text.cat_extended(&wide, Some(b'?'))`.
    pub fn cat_extended(&self, text: &ExtendedString, replacement: Option<u8>) -> Self {
        let mut result = self.clone();
        result.assign_cat_extended(text, replacement);
        result
    }
    /// `Prepend`: inserts the whole of `other` at the front; `text.prepend(&head)`.
    pub fn prepend(&mut self, other: &Self) {
        self.splice_bytes(0, other.as_bytes());
    }
    /// `AssignCat(const char*, int)` with a pointer into this same string: appends a copy of
    /// the zero-based `source` span; `text.assign_cat_self(1..text.len())`.
    pub fn assign_cat_self(&mut self, source: std::ops::Range<usize>) {
        let copy = self.as_bytes()[source].to_vec();
        self.append_bytes(&copy);
    }
}

/// An operand of [`AsciiString::cat`] and `&AsciiString + operand`; each impl is one
/// `Cat` overload. `text.cat(7_u8)` appends byte 7, `text.cat(12)` appends `"12"`.
pub trait CatOperand {
    /// Returns `base` followed by this operand; `CatOperand::cat_onto(b'!', &text)`.
    fn cat_onto(self, base: &AsciiString) -> AsciiString;
}
impl CatOperand for u8 {
    fn cat_onto(self, base: &AsciiString) -> AsciiString {
        base.cat_char(self)
    }
}
/// Operands whose `Cat` overload equals `AssignCat` on a copy.
macro_rules! cat_through_assign {
    ($($operand:ty),*) => {$(
        impl CatOperand for $operand {
            fn cat_onto(self, base: &AsciiString) -> AsciiString {
                let mut result = base.clone();
                result.assign_cat(self);
                result
            }
        }
    )*};
}
cat_through_assign!(
    &str,
    &[u8],
    &AsciiString,
    AsciiString,
    i32,
    f64,
    &ExtendedString
);
impl<const N: usize> CatOperand for &[u8; N] {
    fn cat_onto(self, base: &AsciiString) -> AsciiString {
        self.as_slice().cat_onto(base)
    }
}
impl<T: CatOperand> std::ops::Add<T> for &AsciiString {
    type Output = AsciiString;
    fn add(self, other: T) -> AsciiString {
        self.cat(other)
    }
}
impl<T: Into<AsciiString>> std::ops::AddAssign<T> for AsciiString {
    fn add_assign(&mut self, other: T) {
        self.assign_cat(other);
    }
}
