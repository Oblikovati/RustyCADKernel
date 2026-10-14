// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    super::{RootIsolation, RootIsolationError},
    ResidualError,
};
use super::types::*;
use std::fmt;

/// Stage responsible for a typed root-policy or completeness failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierRegularityStage {
    /// Requested root policy and closed native domain.
    Policy,
    /// Rational denominator root freedom.
    Denominator,
    /// Complete squared-speed numerator isolation.
    SquaredSpeed,
}
/// No successful partial regularity proof accompanies these causes.
#[derive(Debug)]
pub enum BezierRegularityCause {
    /// Unsupported regularity behavior version; expected 1.
    Version(u32),
    /// Closed native interval must be finite, ordered and within `[0,1]`.
    Domain([f64; 2]),
    /// Stored degree exceeds its explicit pre-expansion limit, regardless of reduced degree.
    StoredDegree {
        /// Supplied pole count minus one.
        actual: usize,
        /// Requested stored-degree admission cap.
        limit: u16,
    },
    /// Original shape, finite-coefficient or exact-construction cause.
    Coefficients(ResidualError),
    /// Original root error and its operation stage.
    Roots {
        /// Failed proof stage.
        stage: BezierRegularityStage,
        /// Unmodified shared root-engine cause.
        cause: RootIsolationError,
    },
    /// All denominator coefficients are exactly zero; no rational curve exists.
    ZeroDenominator,
    /// A denominator zero makes this rational representation undefined, even if factors cancel.
    DenominatorZeros(RootIsolation),
}
/// Owned source and typed refusal; completed work is retained without a partial success.
#[derive(Debug)]
pub struct BezierRegularityFailure {
    pub(super) source: Box<BezierRegularitySource>,
    pub(super) cause: BezierRegularityCause,
    pub(super) bisections: u16,
}
impl BezierRegularityFailure {
    /// Borrow original inputs and unchanged policy; example: `failure.source()`.
    pub fn source(&self) -> &BezierRegularitySource {
        &self.source
    }
    /// Read the original typed cause; example: `failure.cause()`.
    pub fn cause(&self) -> &BezierRegularityCause {
        &self.cause
    }
    /// Read subdivisions completed before failure; example: `failure.bisections()`.
    pub fn bisections(&self) -> u16 {
        self.bisections
    }
    /// Distinguish invalid, declined, cancelled and incomplete; example: `failure.completion()`.
    pub fn completion(&self) -> BezierRegularityCompletion {
        self.cause.completion()
    }
}
impl BezierRegularityCause {
    fn completion(&self) -> BezierRegularityCompletion {
        use BezierRegularityCompletion::*;
        match self {
            Self::Version(_) | Self::Domain(_) => Invalid,
            Self::StoredDegree { .. } | Self::ZeroDenominator | Self::DenominatorZeros(_) => {
                Declined
            }
            Self::Roots { cause, .. } | Self::Coefficients(ResidualError::Roots(cause)) => {
                root_completion(cause)
            }
            Self::Coefficients(_) => Invalid,
        }
    }
}
fn root_completion(cause: &RootIsolationError) -> BezierRegularityCompletion {
    use BezierRegularityCompletion::*;
    match cause {
        RootIsolationError::Cancelled => Cancelled,
        RootIsolationError::Degree { .. } => Declined,
        RootIsolationError::SubdivisionLimit { .. } | RootIsolationError::Variation { .. } => {
            Incomplete
        }
        _ => Invalid,
    }
}
impl fmt::Display for BezierRegularityCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version(version) => write!(f, "regularity version {version}; expected 1"),
            Self::Domain(interval) => write!(
                f,
                "regularity interval {interval:?}; expected finite 0 <= first <= last <= 1"
            ),
            Self::StoredDegree { actual, limit } => write!(
                f,
                "stored degree {actual}; expected at most {limit} before power expansion"
            ),
            Self::Coefficients(cause) => write!(
                f,
                "regularity coefficients {cause:?}; expected nonempty poles, finite placement and matching finite signed weights"
            ),
            Self::Roots { stage, cause } => write!(f, "regularity {stage:?}: {cause}"),
            Self::ZeroDenominator => write!(
                f,
                "identically zero denominator; expected no denominator zeros on the closed interval"
            ),
            Self::DenominatorZeros(roots) => write!(
                f,
                "denominator zeros {:?}; expected a complete empty isolation",
                roots.roots()
            ),
        }
    }
}
impl fmt::Display for BezierRegularityFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bézier regularity curve {}, completed subdivisions {}: {}",
            self.source.curve_id(),
            self.bisections,
            self.cause
        )
    }
}
impl std::error::Error for BezierRegularityCause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Coefficients(cause) => Some(cause),
            Self::Roots { cause, .. } => Some(cause),
            _ => None,
        }
    }
}
impl std::error::Error for BezierRegularityFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
