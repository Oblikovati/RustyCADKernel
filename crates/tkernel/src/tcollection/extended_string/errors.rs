//! Exception construction and index arithmetic shared by the ExtendedString members.
use crate::standard::{Failure, exceptions};

/// `Standard_OutOfRange` carrying OCCT's message, then the offending value and the expected
/// shape; `out_of_range("…::Trunc : parameter 'theHowMany'", format_args!("theHowMany 9; expected 0..=5"))`.
pub(super) fn out_of_range(occt_message: &str, detail: std::fmt::Arguments<'_>) -> Failure {
    exceptions::out_of_range(&format!("{occt_message} ({detail})"))
}

/// `Standard_NullObject` for a null pointer argument; `null_object("…: null parameter ")`.
pub(super) fn null_object(occt_message: &str, detail: std::fmt::Arguments<'_>) -> Failure {
    exceptions::null_object(&format!("{occt_message} ({detail})"))
}

/// Zero-based offset of a one-based index that the caller already checked is >= 1;
/// `offset(1) == 0`.
pub(super) fn offset(index: i32) -> usize {
    debug_assert!(index >= 1, "one-based index {index}; expected >= 1");
    (index - 1) as usize
}

/// The signed length OCCT stores in `myLength`; strings longer than `i32::MAX` code units
/// cannot be built by OCCT, so the port does not model them; `int_length(5) == 5`.
pub(super) fn int_length(length: usize) -> i32 {
    length as i32
}

/// Borrows the first `length` units of a nullable (pointer, length) argument.
///
/// # Panics
/// When the slice is shorter than `length`: OCCT would read past the caller's buffer,
/// which is undefined behavior rather than an exception.
pub(super) fn leading_units(units: &[u16], length: usize) -> &[u16] {
    units.get(..length).unwrap_or_else(|| {
        panic!(
            "code-unit span of {} units; expected at least the {length} units the length names",
            units.len()
        )
    })
}
