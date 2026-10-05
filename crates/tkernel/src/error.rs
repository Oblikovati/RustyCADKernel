//! Error transport for the currently ported OCCT operations.

/// OCCT exception categories used by the first foundation APIs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// `Standard_RangeError`.
    Range,
    /// `Quantity_PeriodDefinitionError`.
    PeriodDefinition,
    /// `Quantity_DateDefinitionError`.
    DateDefinition,
    /// An operation would overflow OCCT's signed 32-bit seconds representation.
    /// C++ signed overflow has no defined result to preserve.
    NumericOverflow,
}

/// A typed error preserving the upstream exception category and message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    message: &'static str,
}

impl Error {
    pub(crate) const fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self { kind, message }
    }

    /// The upstream exception category, or a defined Rust overflow error.
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The exception message, without the type name.
    pub const fn message(&self) -> &'static str {
        self.message
    }

    /// The OCCT exception name corresponding to this error.
    pub const fn exception_type(&self) -> &'static str {
        match self.kind {
            ErrorKind::Range => "Standard_RangeError",
            ErrorKind::PeriodDefinition => "Quantity_PeriodDefinitionError",
            ErrorKind::DateDefinition => "Quantity_DateDefinitionError",
            ErrorKind::NumericOverflow => "Standard_Overflow",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.exception_type(), self.message)
    }
}

impl std::error::Error for Error {}

impl From<Error> for crate::standard::Failure {
    /// Preserves the exception name and C-string message; [`Error`] has no original native stack trace.
    ///
    /// ```
    /// use rustycad_tkernel::quantity::Date;
    /// use rustycad_tkernel::standard::Failure;
    /// let failure = Failure::from(Date::new(0, 1, 2000, 0, 0, 0, 0, 0).unwrap_err());
    /// assert!(failure.is_kind("Standard_DomainError"));
    /// ```
    fn from(error: Error) -> Self {
        Self::with_message(error.exception_type(), error.message())
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, ErrorKind};
    use crate::standard::Failure;

    #[test]
    fn error_conversion_stops_at_the_native_c_string_terminator() {
        let error = Error::new(ErrorKind::DateDefinition, "before\0after");
        let failure = Failure::from(error);
        assert_eq!(failure.message(), "before");
    }
}

pub(crate) fn seconds_i32(value: i64) -> Result<i32, Error> {
    i32::try_from(value).map_err(|_| Error::new(ErrorKind::NumericOverflow, "32-bit time overflow"))
}
