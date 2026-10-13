// SPDX-License-Identifier: GPL-2.0-only
use std::fmt;

/// A root query that did not produce a complete certified root list.
#[derive(Clone, Debug, PartialEq)]
pub enum RootIsolationError {
    /// The supplied work policy has an unsupported version.
    OptionsVersion(u32),
    /// A supplied coefficient in the selected basis is not finite.
    Coefficient {
        /// Zero-based position in the supplied coefficient array.
        index: usize,
        /// Rejected nonfinite value.
        value: f64,
    },
    /// Bounds must be finite and ordered.
    Interval([f64; 2]),
    /// The requested approximation width must be finite and positive.
    Width(f64),
    /// A zero polynomial has infinitely many roots, not an empty root list.
    ZeroPolynomial {
        /// Number of supplied zero coefficients.
        coefficients: usize,
    },
    /// The true degree exceeds the caller's admission limit.
    Degree {
        /// True degree after removing leading zeros.
        actual: usize,
        /// Caller-supplied maximum degree.
        limit: u16,
    },
    /// The caller cancelled; no partial list is returned.
    Cancelled,
    /// Subdivision cannot finish within the caller's work limit.
    SubdivisionLimit {
        /// Exhausted caller-supplied subdivision budget.
        limit: u16,
    },
    /// Exact endpoint variation violated the root-count invariant.
    Variation {
        /// Right-hand variation at the lower endpoint.
        lower: usize,
        /// Right-hand variation at the upper endpoint.
        upper: usize,
    },
}
impl fmt::Display for RootIsolationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}; expected {}", self.problem(), self.expected())
    }
}
impl RootIsolationError {
    fn problem(&self) -> String {
        match self {
            Self::OptionsVersion(version) => format!("root policy version {version}"),
            Self::Coefficient { index, value } => format!("coefficient {index} is {value}"),
            Self::Interval(bounds) => format!("root interval {bounds:?}"),
            Self::Width(width) => format!("root width {width}"),
            Self::ZeroPolynomial { coefficients } => {
                format!("zero polynomial with {coefficients} coefficients")
            }
            Self::Degree { actual, limit } => {
                format!("polynomial degree {actual}, caller limit {limit}")
            }
            Self::Cancelled => "root isolation cancelled".into(),
            Self::SubdivisionLimit { limit } => format!("root subdivision limit {limit} exhausted"),
            Self::Variation { lower, upper } => format!("root variations {lower}, {upper}"),
        }
    }
    fn expected(&self) -> &'static str {
        match self {
            Self::OptionsVersion(_) => "root policy version 1",
            Self::Coefficient { .. } => "finite binary64",
            Self::Interval(_) => "finite lower <= upper",
            Self::Width(_) => "finite positive approximation width",
            Self::ZeroPolynomial { .. } => "a nonzero polynomial with finitely many roots",
            Self::Degree { .. } => "degree within the caller limit",
            Self::Cancelled => "completed classification",
            Self::SubdivisionLimit { .. } => "complete isolation within the caller limit",
            Self::Variation { .. } => "nonincreasing variation on ordered bounds",
        }
    }
}
impl std::error::Error for RootIsolationError {}

pub(super) fn poll(cancelled: &mut impl FnMut() -> bool) -> Result<(), RootIsolationError> {
    if cancelled() {
        return Err(RootIsolationError::Cancelled);
    }
    Ok(())
}
