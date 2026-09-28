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

pub(crate) fn seconds_i32(value: i64) -> Result<i32, Error> {
    i32::try_from(value).map_err(|_| Error::new(ErrorKind::NumericOverflow, "32-bit time overflow"))
}
