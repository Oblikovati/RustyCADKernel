// SPDX-License-Identifier: GPL-2.0-only
use super::super::{super::RootIsolationError, ResidualError};
use super::{PolynomialSideError, PolynomialSideOptions};

/// A failed or incomplete conditional scalar operation; never a successful partial integral.
/// Example: `matches!(*error.cause, PolynomialSideFailure::Cancelled)`.
#[derive(Debug)]
pub enum PolynomialSideFailure {
    /// Unsupported version, degree/work cap or nonfinite/negative error limit.
    Policy(PolynomialSideOptions),
    /// Supplied stored degree exceeds the requested maximum.
    Degree {
        /// Supplied stored polynomial degree.
        actual: usize,
        /// Requested admission maximum.
        limit: u16,
    },
    /// All explicit weights, including equal weights, are outside this polynomial domain.
    RationalWeights {
        /// Number of explicit weights supplied.
        count: usize,
    },
    /// Original typed metric input failure.
    Metric(Box<ResidualError>),
    /// The caller cancelled at an observed deterministic checkpoint.
    Cancelled,
    /// Requested work was exhausted before the certified error met the unchanged limit.
    WorkLimit {
        /// Requested maximum completed subdivisions.
        requested: u32,
        /// Upward-rounded certified error at exhaustion.
        achieved_error: f64,
        /// Unchanged requested absolute area error limit.
        error_limit: f64,
    },
    /// An exact nonnegative bound cannot be enclosed in finite binary64.
    Unrepresentable {
        /// Exact bound whose finite outward representation failed.
        quantity: &'static str,
        /// Offending exact rational value, before any overflow or underflow.
        exact_value: String,
    },
}
impl From<ResidualError> for PolynomialSideFailure {
    fn from(cause: ResidualError) -> Self {
        if matches!(cause, ResidualError::Roots(RootIsolationError::Cancelled)) {
            return Self::Cancelled;
        }
        Self::Metric(Box::new(cause))
    }
}
impl std::fmt::Display for PolynomialSideFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let expected = self.expected();
        write!(formatter, "{self:?}; expected {expected}")
    }
}
impl PolynomialSideFailure {
    fn expected(&self) -> &'static str {
        match self {
            Self::Policy(_) => "version 1, degree <=16, splits <=65536, finite area error >=0",
            Self::Degree { .. } => "stored degree within requested maximum",
            Self::RationalWeights { .. } => "polynomial curve with omitted weights",
            Self::Metric(_) => "finite valid supplied affine polynomial coefficients",
            Self::Cancelled => "uncancelled completion",
            Self::WorkLimit { .. } => "certified error within requested cap before work exhaustion",
            Self::Unrepresentable { .. } => "finite binary64 certified enclosure",
        }
    }
}
impl std::fmt::Display for PolynomialSideError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "polynomial extrusion-side integral v1, supplied curve: {}",
            self.cause
        )
    }
}
impl std::error::Error for PolynomialSideFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Metric(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}
impl std::error::Error for PolynomialSideError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        let cause = self.cause.as_ref();
        Some(cause)
    }
}
