// SPDX-License-Identifier: GPL-2.0-only
use super::super::{super::RootIsolationError, ResidualError};
use super::{PolynomialIntegralOperation, PolynomialIntegralOptions};

/// Incomplete integration retains its operation, offending occurrence and typed cause.
#[derive(Debug)]
pub struct PolynomialIntegralError {
    /// Requested scalar operation, never an assertion of valid geometry.
    pub operation: PolynomialIntegralOperation,
    /// Original policy for diagnosing the failed request; no effective clamp is applied.
    pub requested_options: PolynomialIntegralOptions,
    /// Diagnostic position in the caller's ordered slice, not an entity identity.
    pub curve_index: Option<usize>,
    /// Why no complete certified scalar report was published.
    pub cause: PolynomialIntegralFailure,
}
/// Unsupported domains and unmet bounds are incomplete, never a successful partial integral.
#[derive(Debug)]
pub enum PolynomialIntegralFailure {
    /// Unsupported version, work limit, or nonfinite/negative error cap.
    Policy(PolynomialIntegralOptions),
    /// Empty boundary input is distinct from a completed zero integral.
    EmptyBoundary,
    /// No occurrences are truncated when the requested count limit is exceeded.
    CurveCount {
        /// Supplied occurrence count.
        actual: usize,
        /// Requested maximum.
        limit: u16,
    },
    /// Stored degree exceeds the requested admission limit, even if leading terms cancel.
    Degree {
        /// Supplied stored degree.
        actual: usize,
        /// Requested maximum.
        limit: u16,
    },
    /// All supplied rational weights, including equal weights, are outside version 1.
    RationalWeights {
        /// Number of supplied weights.
        count: usize,
    },
    /// Original metric input or cancellation cause; no string-only erasure.
    Metric(Box<ResidualError>),
    /// Exact value cannot be enclosed by finite binary64 endpoints.
    Unrepresentable {
        /// Exact rational value in numerator/denominator diagnostic form.
        exact_value: String,
    },
    /// Certified rounding bound exceeds the unchanged requested cap.
    ErrorLimit {
        /// Requested absolute error cap.
        requested: f64,
        /// Upward-rounded achieved absolute error bound.
        achieved: f64,
    },
}
impl From<ResidualError> for PolynomialIntegralFailure {
    fn from(cause: ResidualError) -> Self {
        Self::Metric(Box::new(cause))
    }
}
impl From<RootIsolationError> for PolynomialIntegralFailure {
    fn from(cause: RootIsolationError) -> Self {
        Self::Metric(Box::new(ResidualError::Roots(cause)))
    }
}
impl std::fmt::Display for PolynomialIntegralError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:?} occurrence {:?}: {}",
            self.operation, self.curve_index, self.cause
        )
    }
}
impl std::fmt::Display for PolynomialIntegralFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}; expected {}", self.expected())
    }
}
impl PolynomialIntegralFailure {
    fn expected(&self) -> &'static str {
        match self {
            Self::Policy(_) => "version 1, degree <=16, 1<=curves<=64, finite error cap >=0",
            Self::EmptyBoundary => "at least one supplied occurrence",
            Self::CurveCount { .. } => "occurrence count within requested maximum",
            Self::Degree { .. } => "stored degree within requested maximum",
            Self::RationalWeights { .. } => "polynomial curve with omitted weights",
            Self::Metric(_) => "valid finite metric inputs and uncancelled completion",
            Self::Unrepresentable { .. } => "finite binary64 enclosure endpoints",
            Self::ErrorLimit { .. } => "certified error bound within requested cap",
        }
    }
}
impl std::error::Error for PolynomialIntegralError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
impl std::error::Error for PolynomialIntegralFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Metric(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}
