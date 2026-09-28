//! Errors for checked array construction, assignment and indexing.
use crate::standard::Failure;
use std::fmt;

/// Reports a failed array operation with its supplied and expected range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrayError {
    /// Inclusive bounds cannot describe a nonnegative, addressable size.
    Bounds {
        /// Requested lower bound.
        lower: i32,
        /// Requested upper bound.
        upper: i32,
    },
    /// An index lies outside the accessible range.
    Index {
        /// Requested index.
        index: i64,
        /// Inclusive lower bound.
        lower: i64,
        /// Inclusive upper bound.
        upper: i64,
    },
    /// CopyValues requires identical rectangular dimensions.
    ShapeMismatch {
        /// Destination rows and columns.
        expected: (usize, usize),
        /// Source rows and columns.
        actual: (usize, usize),
    },
    /// CopyValues requires identical lengths.
    DimensionMismatch {
        /// Destination length.
        expected: usize,
        /// Source length.
        actual: usize,
    },
}

impl fmt::Display for ArrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bounds { lower, upper } => write!(
                f,
                "invalid bounds [{lower}, {upper}]: expected upper >= lower - 1"
            ),
            Self::Index {
                index,
                lower,
                upper,
            } => write!(f, "index {index}: expected [{lower}, {upper}]"),
            Self::ShapeMismatch { expected, actual } => {
                write!(f, "shape {actual:?}: expected {expected:?}")
            }
            Self::DimensionMismatch { expected, actual } => {
                write!(f, "length {actual}: expected {expected}")
            }
        }
    }
}

impl std::error::Error for ArrayError {}

impl ArrayError {
    /// The exception a native `NCollection_Array1` raises for this error, prefixed with
    /// the member that indexed it;
    /// `ArrayError::Bounds { lower: 2, upper: 0 }.into_failure("NCollection_HArray1")`.
    pub fn into_failure(self, member: &str) -> Failure {
        let kind = match self {
            Self::Bounds { .. } => "Standard_RangeError",
            Self::Index { .. } => "Standard_OutOfRange",
            Self::ShapeMismatch { .. } | Self::DimensionMismatch { .. } => {
                "Standard_DimensionMismatch"
            }
        };
        Failure::with_type(kind, &format!("{member}: {self}"), "")
    }
}
