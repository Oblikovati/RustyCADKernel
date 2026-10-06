//! AssignCat, Cat and Prepend.
use super::{
    ExtendedString,
    errors::{leading_units, out_of_range},
    utf,
};
use crate::standard::Failure;
impl ExtendedString {
    /// Appends converted input: a string, a code-unit slice, a byte (`char`), a unit, an
    /// integer (`%d`) or a real (`%g`); zero characters append nothing. `text.assign_cat(42)`.
    pub fn assign_cat(&mut self, input: impl Into<Self>) {
        let input = input.into();
        self.append_units(input.as_utf16());
    }
    /// Appends a bounded null-terminated UTF-16 buffer; `text.assign_cat_c_units(Some(&[65, 0, 66]))`.
    /// A null pointer is represented by `None` and appends nothing.
    pub fn assign_cat_c_units(&mut self, units: Option<&[u16]>) {
        let Some(units) = units else {
            return;
        };
        self.append_units(utf::c_units_prefix(units));
    }
    /// Returns a copy with converted input appended; `text.cat(b'!')`.
    pub fn cat(&self, input: impl Into<Self>) -> Self {
        let mut copy = self.clone();
        copy.assign_cat(input);
        copy
    }
    /// Returns a copy with a bounded null-terminated UTF-16 buffer appended;
    /// `text.cat_c_units(Some(&[65, 0, 66]))`. A null pointer uses `None`.
    pub fn cat_c_units(&self, units: Option<&[u16]>) -> Self {
        let mut copy = self.clone();
        copy.assign_cat_c_units(units);
        copy
    }
    /// Prepends the portion of a bounded null-terminated UTF-16 buffer before its first zero;
    /// `text.prepend_c_units(Some(&[65, 0, 66]))`. A null pointer uses `None`.
    pub fn prepend_c_units(&mut self, units: Option<&[u16]>) {
        let Some(units) = units else {
            return;
        };
        self.prepend(utf::c_units_prefix(units));
    }
    /// Prepends a NUL-terminated suffix of this string, preserving OCCT's aliased-pointer case;
    /// `text.prepend_c_units_from_self(2)?`.
    pub fn prepend_c_units_from_self(&mut self, start: usize) -> Result<(), Failure> {
        let Some(suffix) = self.as_utf16().get(start..) else {
            return Err(out_of_range(
                "TCollection_ExtendedString::Prepend : self code-unit offset",
                format_args!("start {start}; expected 0..={}", self.len()),
            ));
        };
        let prefix_length = utf::c_units_prefix(suffix).len();
        self.prepend_self(start..start + prefix_length);
        Ok(())
    }
    /// The (pointer, length) AssignCat: a null pointer or a length <= 0 appends nothing;
    /// `text.assign_cat_span(Some(&[65, 66]), 1)`.
    pub fn assign_cat_span(&mut self, units: Option<&[u16]>, length: i32) {
        let (Some(units), Ok(count)) = (units, usize::try_from(length)) else {
            return;
        };
        self.append_units(leading_units(units, count));
    }
    /// The (pointer, length) Cat. OCCT builds its result with the (length, filler)
    /// constructor, so a negative total raises Standard_OutOfRange; any other negative
    /// length overruns the native buffer and is refused the same way.
    /// `text.cat_span(Some(&[65]), 1)?`.
    pub fn cat_span(&self, units: Option<&[u16]>, length: i32) -> Result<Self, Failure> {
        let Ok(count) = usize::try_from(length) else {
            return Err(out_of_range(
                "TCollection_ExtendedString : negative length",
                format_args!(
                    "theLength {length} with myLength {}; expected theLength >= 0",
                    self.len()
                ),
            ));
        };
        let mut result = self.clone();
        if count > 0 {
            let units = units.unwrap_or_else(|| {
                panic!("null code-unit pointer with length {length}; expected a buffer")
            });
            result.append_units(leading_units(units, count));
        }
        Ok(result)
    }
    /// Inserts a string or code-unit slice at the beginning; `text.prepend(&prefix)`.
    pub fn prepend(&mut self, input: impl AsRef<[u16]>) {
        let units = input.as_ref();
        if units.is_empty() {
            return;
        }
        self.ensure_owned_terminator();
        self.terminated.splice(0..0, units.iter().copied());
    }
    /// The (pointer, length) Prepend: a null pointer or a length <= 0 changes nothing;
    /// `text.prepend_span(Some(&[65]), 1)`.
    pub fn prepend_span(&mut self, units: Option<&[u16]>, length: i32) {
        let (Some(units), Ok(count)) = (units, usize::try_from(length)) else {
            return;
        };
        self.prepend(leading_units(units, count));
    }
    /// Appends a range of this string's own units (OCCT's aliased-pointer AssignCat);
    /// `text.assign_cat_self(0..text.len())`.
    pub fn assign_cat_self(&mut self, range: std::ops::Range<usize>) {
        let copy = self.as_utf16()[range].to_vec();
        self.append_units(&copy);
    }
    /// Prepends a range of this string's own units (OCCT's aliased-pointer Prepend);
    /// `text.prepend_self(5..10)`.
    pub fn prepend_self(&mut self, range: std::ops::Range<usize>) {
        let copy = self.as_utf16()[range].to_vec();
        self.prepend(copy);
    }
    /// Appends units before the terminator.
    fn append_units(&mut self, units: &[u16]) {
        if units.is_empty() {
            return;
        }
        self.ensure_owned_terminator();
        let end = self.len();
        self.terminated.splice(end..end, units.iter().copied());
    }
}
