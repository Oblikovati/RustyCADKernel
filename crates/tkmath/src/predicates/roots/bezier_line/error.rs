// SPDX-License-Identifier: GPL-2.0-only
use super::super::{RootIsolation, RootIsolationError};
use crate::predicates::NonFiniteCoordinate;
use std::fmt;
/// Source owning a rejected coordinate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierInput {
    /// Infinite line endpoints.
    Line,
    /// Bézier control points.
    Control,
}
/// Stage owning an incomplete root computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BezierStage {
    /// Input policy or exact projection.
    Input,
    /// Rational denominator regularity.
    Denominator,
    /// Curve-to-line contact numerator.
    Numerator,
}
/// A query that produced no completed contact classification.
#[derive(Clone, Debug, PartialEq)]
pub enum BezierLineError {
    /// A curve requires at least one supplied control point.
    EmptyControls,
    /// The supplied weight count differs from the control count.
    WeightCount {
        /// Supplied control count.
        controls: usize,
        /// Supplied weight count.
        weights: usize,
    },
    /// A nonfinite coordinate with its input owner and precise position.
    Coordinate {
        /// Input owner.
        input: BezierInput,
        /// Original coordinate failure, including the offending value.
        cause: NonFiniteCoordinate,
    },
    /// A nonfinite weight.
    Weight {
        /// Zero-based control position.
        index: usize,
        /// Offending weight.
        value: f64,
    },
    /// Coincident endpoints do not define a line.
    DegenerateLine([[f64; 2]; 2]),
    /// The denominator is identically zero.
    ZeroDenominator,
    /// Denominator zeros on the closed query interval; no contact result is valid.
    DenominatorRoots(RootIsolation),
    /// A typed root-policy, work-limit, or cancellation failure.
    Root {
        /// Stage where completion failed.
        stage: BezierStage,
        /// Original failure.
        cause: RootIsolationError,
    },
}
impl fmt::Display for BezierLineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}; expected {}",
            self.problem(),
            self.expected()
        )
    }
}
impl BezierLineError {
    fn problem(&self) -> String {
        match self {
            Self::EmptyControls => "Bézier control count 0".into(),
            Self::WeightCount { controls, weights } => {
                format!("{controls} controls, {weights} weights")
            }
            Self::Coordinate { input, cause } => format!("{input:?} {cause}"),
            Self::Weight { index, value } => format!("Bézier weight {index} is {value}"),
            Self::DegenerateLine(line) => format!("line endpoints {line:?}"),
            Self::ZeroDenominator => "identically zero Bézier denominator".into(),
            Self::DenominatorRoots(roots) => format!("denominator zeros {:?}", roots.roots()),
            Self::Root { stage, cause } => format!("Bézier {stage:?}: {cause}"),
        }
    }
    fn expected(&self) -> &'static str {
        match self {
            Self::EmptyControls => "at least one control point",
            Self::WeightCount { .. } => "one weight per control point",
            Self::Coordinate { .. } | Self::Weight { .. } => "finite binary64 values",
            Self::DegenerateLine(_) => "two distinct line endpoints",
            Self::ZeroDenominator | Self::DenominatorRoots(_) => {
                "a nonzero denominator throughout the closed interval"
            }
            Self::Root { .. } => "a complete exact parameter classification",
        }
    }
}
impl std::error::Error for BezierLineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Root { cause, .. } => Some(cause),
            Self::Coordinate { cause, .. } => Some(cause),
            _ => None,
        }
    }
}
