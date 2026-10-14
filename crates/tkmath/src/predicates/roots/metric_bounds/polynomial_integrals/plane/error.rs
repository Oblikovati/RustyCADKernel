// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::ResidualError;
use super::*;
use crate::predicates::roots::RootIsolationError;

/// Incomplete conditional world-plane measurement; no partial scalar is published.
/// The type identifies the operation; boundary causes retain the offending curve index.
#[derive(Debug)]
pub struct PolynomialPlaneError {
    /// Requested admission limits and world-area error cap, never widened.
    pub requested_options: PolynomialIntegralOptions,
    /// Typed original cause or failed algebraic enclosure/budget.
    pub cause: PolynomialPlaneFailure,
}
/// Why the supplied conditional world-plane scalar could not be certified.
#[derive(Debug)]
pub enum PolynomialPlaneFailure {
    /// Exact native-boundary integration failed, preserving its policy and curve index.
    Boundary(Box<PolynomialIntegralError>),
    /// Placed support admission or cancellation failed; original metric context is retained.
    Metric(Box<ResidualError>),
    /// Exact nonnegative squared measure whose square root has no finite enclosure.
    UnrepresentableSquaredMeasure(String),
    /// Directed enclosure width exceeds the unchanged world-area budget.
    ErrorLimit {
        /// Original finite absolute error cap.
        requested: f64,
        /// Upward-rounded achieved enclosure width.
        achieved: f64,
    },
}
impl From<PolynomialIntegralError> for PolynomialPlaneFailure {
    fn from(cause: PolynomialIntegralError) -> Self {
        // Native integration retains its own occurrence and policy context.
        Self::Boundary(Box::new(cause))
    }
}
impl From<ResidualError> for PolynomialPlaneFailure {
    fn from(cause: ResidualError) -> Self {
        // Metric admission preserves the offending placed coefficient.
        Self::Metric(Box::new(cause))
    }
}
impl From<RootIsolationError> for PolynomialPlaneFailure {
    fn from(cause: RootIsolationError) -> Self {
        // Cancellation retains the same root-error chain as existing metric arithmetic.
        Self::Metric(Box::new(ResidualError::Roots(cause)))
    }
}
impl std::fmt::Display for PolynomialPlaneError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "conditional world-plane measure {:?}: {}",
            self.requested_options, self.cause
        )
    }
}
impl std::fmt::Display for PolynomialPlaneFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Diagnostics pair the offending payload with the refused admission condition.
        write!(formatter, "{self:?}; expected {}", self.expected())
    }
}
impl PolynomialPlaneFailure {
    fn expected(&self) -> &'static str {
        match self {
            Self::Boundary(_) => "complete admitted native polynomial boundary integral",
            Self::Metric(_) => "finite placed support and uncancelled completion",
            Self::UnrepresentableSquaredMeasure(_) => "finite square-root enclosure endpoints",
            Self::ErrorLimit { .. } => "certified width within requested world-area cap",
        }
    }
}
impl std::error::Error for PolynomialPlaneError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Preserve typed causes for callers inspecting incomplete arithmetic.
        Some(&self.cause)
    }
}
impl std::error::Error for PolynomialPlaneFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Boundary(cause) => Some(cause.as_ref()),
            Self::Metric(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}
