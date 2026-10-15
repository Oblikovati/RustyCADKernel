// SPDX-License-Identifier: GPL-2.0-only
use super::super::{super::RootIsolationError, PolynomialIntegralOperation, ResidualError};
use super::RationalIntegralOptions;

/// Incomplete integration retains its operation, offending occurrence and typed cause.
#[derive(Debug)]
pub struct RationalIntegralError {
    /// Requested scalar operation, never an assertion of valid geometry.
    pub operation: PolynomialIntegralOperation,
    /// Original policy for diagnosing the failed request; no effective clamp is applied.
    pub requested_options: RationalIntegralOptions,
    /// Diagnostic position in the caller's ordered slice, not an entity identity.
    pub curve_index: Option<usize>,
    /// Why no complete certified scalar report was published.
    pub cause: RationalIntegralFailure,
}

/// Unsupported domains and unmet bounds are incomplete, never a partial integral.
#[derive(Debug)]
pub enum RationalIntegralFailure {
    /// Unsupported version, work limit, or nonfinite/negative error cap.
    Policy(RationalIntegralOptions),
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
    /// A subinterval denominator lost strict Bernstein positivity (never for positive weights).
    DenominatorSign {
        /// Lower parameter endpoint of the refused subinterval.
        start: f64,
        /// Upper parameter endpoint of the refused subinterval.
        end: f64,
    },
    /// The curve needed more subintervals than allowed to meet its error share.
    WorkLimit {
        /// Requested per-curve subinterval maximum.
        limit: u32,
        /// Downward-rounded parameter length of `[0,1]` already certified within its share.
        certified_length: f64,
    },
    /// ADR 0066 part C: a face loop's certified integral does not exclude zero.
    IndeterminateOrientation {
        /// Flattened index of the loop's first curve (outer first, then holes).
        first_curve: usize,
    },
    /// Original metric input or cancellation cause; no string-only erasure.
    Metric(Box<ResidualError>),
    /// Exact enclosure endpoint cannot be represented by finite binary64 values.
    Unrepresentable {
        /// Exact rational endpoint in numerator/denominator diagnostic form.
        exact_value: String,
    },
    /// Certified error bound exceeds the unchanged requested cap.
    ErrorLimit {
        /// Requested absolute error cap.
        requested: f64,
        /// Upward-rounded achieved absolute error bound.
        achieved: f64,
    },
}
impl From<ResidualError> for RationalIntegralFailure {
    fn from(cause: ResidualError) -> Self {
        Self::Metric(Box::new(cause))
    }
}
impl From<RootIsolationError> for RationalIntegralFailure {
    fn from(cause: RootIsolationError) -> Self {
        Self::Metric(Box::new(ResidualError::Roots(cause)))
    }
}
impl std::fmt::Display for RationalIntegralError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:?} occurrence {:?}: {}",
            self.operation, self.curve_index, self.cause
        )
    }
}
impl std::fmt::Display for RationalIntegralFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}; expected {}", self.expected())
    }
}
impl RationalIntegralFailure {
    fn expected(&self) -> &'static str {
        match self {
            Self::Policy(_) => {
                "version 1, degree <=16, 1<=curves<=64, 1<=subintervals<=65536, finite cap >=0"
            }
            Self::EmptyBoundary => "at least one supplied occurrence",
            Self::CurveCount { .. } => "occurrence count within requested maximum",
            Self::Degree { .. } => "stored degree within requested maximum",
            Self::DenominatorSign { .. } => "strictly positive Bernstein denominator",
            Self::WorkLimit { .. } => "error share met within the subinterval maximum",
            Self::IndeterminateOrientation { .. } => "loop integral enclosure excluding zero",
            Self::Metric(_) => "valid finite metric inputs and uncancelled completion",
            Self::Unrepresentable { .. } => "finite binary64 enclosure endpoints",
            Self::ErrorLimit { .. } => "certified error bound within requested cap",
        }
    }
}
impl std::error::Error for RationalIntegralError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
impl std::error::Error for RationalIntegralFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Metric(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}
