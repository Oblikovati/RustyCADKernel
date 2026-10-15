// SPDX-License-Identifier: GPL-2.0-only
//! Named noncompletion; no failure carries a successful partial contact list.
use super::super::RootIsolationError;
use std::fmt;

/// Pipeline stage that owned the failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SketchIncidenceStage {
    /// Options, source or target admission.
    Input,
    /// Exact equation construction.
    Equation,
    /// Complete root isolation of a derived equation.
    Isolation,
    /// World point construction to the requested error.
    Construction,
    /// Exact position, sector and transition classification.
    Classification,
    /// Exact ordering of contacts along the source line.
    Ordering,
    /// Root selection for an edit target.
    Selection,
}
/// Typed cause, retaining the offending value or exhausted limit.
#[derive(Clone, Debug, PartialEq)]
pub enum SketchIncidenceCause {
    /// Unsupported options version; expected 1.
    OptionsVersion(u32),
    /// Rejected world error limit; expected finite and positive.
    WorldErrorLimit(f64),
    /// A supplied scalar or coordinate is not finite.
    NonFinite {
        /// Input field name.
        field: &'static str,
        /// Zero-based element index within the field.
        index: usize,
        /// Rejected value.
        value: f64,
    },
    /// A radius or weight is not strictly positive.
    NonPositive {
        /// Input field name.
        field: &'static str,
        /// Zero-based element index within the field.
        index: usize,
        /// Rejected value.
        value: f64,
    },
    /// Two points that must differ are equal, or a direction is zero.
    Degenerate {
        /// Input field name.
        field: &'static str,
    },
    /// Sector boundaries name the same ray.
    CoincidentSector {
        /// Supplied start direction.
        start: [f64; 2],
        /// Supplied end direction.
        end: [f64; 2],
    },
    /// Spline counts do not describe one B-spline.
    SplineShape {
        /// Supplied degree.
        degree: usize,
        /// Supplied pole count.
        poles: usize,
        /// Supplied flat knot count.
        knots: usize,
        /// Supplied weight count, if any.
        weights: Option<usize>,
    },
    /// Knots decrease at this index, or an interior knot breaks C0 continuity.
    KnotSequence {
        /// First offending flat knot index.
        index: usize,
    },
    /// Spline bounds are unordered or leave the knot domain.
    SplineBounds {
        /// Supplied bounds.
        bounds: [f64; 2],
        /// Admitted domain `[knots[degree], knots[poles]]`.
        domain: [f64; 2],
    },
    /// Original root isolation, degree or work failure.
    Root(RootIsolationError),
    /// Caller cancellation.
    Cancelled,
    /// The total refinement budget was used before certification completed.
    RefinementExhausted {
        /// Exhausted caller limit.
        limit: u32,
    },
    /// The requested world error cannot be certified by a finite binary64 point.
    ConstructionResolution {
        /// Caller limit; never widened.
        requested: f64,
        /// Last certified bound, or None when no finite enclosure exists.
        achieved: Option<f64>,
    },
    /// A coincident span is not strictly monotone along the line.
    NonMonotoneOverlap {
        /// Target parameter range of the span.
        range: [f64; 2],
    },
    /// An image or elimination resultant vanished identically (a base point).
    DegenerateResultant,
    /// A piece lies on the other curve; version 1 certifies only line-source overlaps.
    UncertifiedOverlap {
        /// Parameter range of the coincident piece; None for an unbounded line.
        range: Option<[f64; 2]>,
    },
    /// Two conics meet tangentially on both half turns of one line through the
    /// source center; version 1 certifies only transversal points there.
    ExceptionalTangency,
    /// A spline-pair fibre is not one certified target point (several target
    /// points, a straight span of degree two or more, or a stationary span).
    ExceptionalFiber {
        /// Source span range of the root.
        range: Option<[f64; 2]>,
    },
    /// A contact at a target interior knot is tangential to a piece, or the source
    /// is stationary or cusped there.
    KnotTangency,
    /// A line source must be bounded: its two points are its ends.
    UnboundedSource,
    /// The requested domain is not defined for this source representation.
    SourceDomain {
        /// Name of the source representation, e.g. `"spline"`.
        source: &'static str,
        /// The refused domain.
        domain: super::input::SketchLineDomain,
    },
    /// An exact invariant failed; reported rather than repaired.
    Invariant(&'static str),
}
/// A request lacking complete certification. Target is None for source or options failures.
#[derive(Clone, Debug, PartialEq)]
pub struct SketchIncidenceFailure {
    pub(super) target: Option<u64>,
    pub(super) stage: SketchIncidenceStage,
    pub(super) cause: SketchIncidenceCause,
}
/// Coarse outcome of a failure; completed empty results are never failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SketchIncidenceCompletion {
    /// Invalid input or options.
    Invalid,
    /// Caller cancellation.
    Cancelled,
    /// Exhausted, unresolved or degenerate certification.
    Incomplete,
}
impl SketchIncidenceFailure {
    pub(super) fn new(stage: SketchIncidenceStage, cause: SketchIncidenceCause) -> Self {
        Self {
            target: None,
            stage,
            cause,
        }
    }
    pub(super) fn for_target(mut self, target: u64) -> Self {
        self.target.get_or_insert(target);
        self
    }
    /// Target occurrence key that failed; example: `failure.target() == Some(7)`.
    pub fn target(&self) -> Option<u64> {
        self.target
    }
    /// Failing stage; example: `failure.stage() == SketchIncidenceStage::Ordering`.
    pub fn stage(&self) -> SketchIncidenceStage {
        self.stage
    }
    /// Typed cause; example: `matches!(failure.cause(), SketchIncidenceCause::Cancelled)`.
    pub fn cause(&self) -> &SketchIncidenceCause {
        &self.cause
    }
    /// Classify the outcome; example: `failure.completion() == SketchIncidenceCompletion::Invalid`.
    pub fn completion(&self) -> SketchIncidenceCompletion {
        use SketchIncidenceCause as Cause;
        match &self.cause {
            Cause::Cancelled | Cause::Root(RootIsolationError::Cancelled) => {
                SketchIncidenceCompletion::Cancelled
            }
            _ if self.stage == SketchIncidenceStage::Input => SketchIncidenceCompletion::Invalid,
            _ => SketchIncidenceCompletion::Incomplete,
        }
    }
}
impl From<RootIsolationError> for SketchIncidenceCause {
    fn from(cause: RootIsolationError) -> Self {
        match cause {
            RootIsolationError::Cancelled => Self::Cancelled,
            other => Self::Root(other),
        }
    }
}
impl fmt::Display for SketchIncidenceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "sketch line incidence {:?} stage, target {:?}: {:?}; expected complete certification",
            self.stage, self.target, self.cause
        )
    }
}
impl std::error::Error for SketchIncidenceFailure {}
