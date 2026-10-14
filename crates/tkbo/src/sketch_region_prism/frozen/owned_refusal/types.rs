// SPDX-License-Identifier: GPL-2.0-only
//! A completed projection owns diagnostic values, never live receipts or address-based identities.
use super::*;
use crate::sketch_region_geometry_capture::CapturedGeometryCause;
use rustycad_tkmath::predicates::roots::metric_bounds::{
    BezierRegularityFailure, BezierRegularityReport, ResidualError,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismRefusalClass {
    Failed,
    Incomplete,
    Cancelled,
}
#[derive(Debug)]
pub(crate) struct OwnedPrismJoinRefusal {
    pub(super) operation: AuthoredPrismId,
    pub(super) phase: PrismProofJoinPhase,
    pub(super) class: PrismRefusalClass,
    pub(super) entity: Option<FrozenPrismKey>,
    pub(super) cause: OwnedPrismJoinCause,
}
pub(crate) enum PrismJoinRefusalProjection {
    Projected(Box<OwnedPrismJoinRefusal>),
    Incomplete(Box<LocalPrismProjectionFailure>),
}
pub(crate) struct LocalPrismProjectionFailure {
    pub(super) class: PrismRefusalClass,
    pub(super) issue: PrismRefusalProjectionIssue,
    pub(super) original: PrismProofJoinFailure,
}
#[derive(Debug)]
pub(crate) struct PrismRefusalProjectionIssue {
    pub(super) path: Vec<PrismRefusalPath>,
    pub(super) cause: PrismRefusalProjectionGap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismRefusalPath {
    Join,
    Embedding,
    Binding,
    Region,
    Source,
    Body,
    Candidate,
    Frozen,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PendingPrismRefusalPayload {
    ClosurePoint,
    CandidateReport,
    CandidateGap,
    CandidateCapturedOperation,
    BodySelection,
    BodyShape,
}
#[derive(Debug)]
pub(crate) enum PrismRefusalProjectionGap {
    /// The operation token must be nonzero before dictionary binding can have meaning.
    InvalidOperation(AuthoredPrismId),
    Operation {
        actual: AuthoredPrismId,
        expected: AuthoredPrismId,
    },
    Original {
        source: u64,
        cause: Box<PrismFreezeCause>,
    },
    UnprojectedDiagnostic {
        stage: Option<crate::sketch_region_validation::ValidationStage>,
        actual: &'static str,
    },
    Pending(PendingPrismRefusalPayload),
}
pub(super) struct RetainedProjection<T> {
    pub issue: PrismRefusalProjectionIssue,
    pub original: T,
}
pub(super) type ProjectionCheck<T, L> = Result<T, Box<RetainedProjection<L>>>;

#[derive(Debug)]
pub(crate) enum OwnedPrismJoinCause {
    MissingClosureQuery,
    ClosureQuery(Box<OwnedPrismSourceCause>),
    ClosureRegion(Box<OwnedPrismRegionCause>),
    Version { actual: u32, expected: u32 },
    Embedding(Box<OwnedPrismEmbeddingFailure>),
    IncompleteCapture(Box<PrismFreezeFailure>),
    FailedCapture(Box<PrismFreezeFailure>),
    Binding(Box<OwnedPrismBindingCause>),
    Link(Box<PrismFreezeCause>),
    Cancelled { freeze_phase: Option<FreezePhase> },
    Stale { representation: &'static str },
}
#[derive(Debug)]
pub(crate) struct OwnedPrismEmbeddingFailure {
    pub requested: PrismEmbeddingOptions,
    pub phase: PrismEmbeddingPhase,
    pub original: Option<FrozenOriginalKey>,
    pub cause: OwnedPrismEmbeddingCause,
}
#[derive(Debug)]
pub(crate) enum OwnedPrismEmbeddingCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Binding(Box<OwnedPrismBindingCause>),
    Regularity(Box<BezierRegularityFailure>),
    NonRegular(Box<BezierRegularityReport>),
    /// The retained curve ID is a census observation, not a persistent entity identity.
    PolynomialRequired {
        curve_id: u64,
        explicit_weights: Vec<f64>,
    },
}
#[derive(Debug)]
pub(crate) enum OwnedPrismBindingCause {
    Region(OwnedPrismRegionCause),
    Classification(Box<OwnedPrismCandidateCause>),
    Chart(PrismSurfaceCause),
    Body(OwnedPrismBodyCause),
    Cancelled,
}
#[derive(Debug)]
pub(crate) enum OwnedPrismCandidateCause {
    Native(Failure),
    Cancelled(crate::sketch_region_candidates::CandidatePhase),
    Numerical(rustycad_tkmath::predicates::roots::bezier_line::BezierWireError),
    CapturedNumerical(rustycad_tkmath::predicates::roots::bezier_line::CapturedBezierWireFailure),
}
#[derive(Debug)]
pub(crate) enum OwnedPrismBodyCause {
    Incomplete,
    Cancelled(PrismPhase),
    Failed(OwnedPrismSweepCause),
}
#[derive(Debug)]
pub(crate) enum OwnedPrismSweepCause {
    Mapped {
        phase: PrismPhase,
        failure: Failure,
    },
    ValidationRejected(crate::sketch_region_validation::ValidationStage),
    InvalidRequest {
        quantity: &'static str,
        actual: Vec<f64>,
    },
    Classification(Box<OwnedPrismCandidateCause>),
    ClassificationProgress {
        failure: Box<Failure>,
        query: Box<OwnedPrismCandidateCause>,
    },
}
#[derive(Debug)]
pub(crate) enum OwnedPrismRegionCause {
    UnavailableCandidates,
    MissingEvidence {
        hypothesis: &'static str,
    },
    Hypothesis {
        name: &'static str,
        actual: usize,
        expected: usize,
    },
    Capture(Arc<PrismSurfaceCause>),
    Mapped(Failure),
}
#[derive(Debug)]
pub(crate) enum OwnedPrismSourceCause {
    Capture(Arc<CapturedGeometryCause>),
    MissingOriginal(FrozenOriginalKey),
    NativeRange {
        original: [f64; 2],
        copied: [f64; 2],
        stored: [f64; 2],
    },
    InvalidSegmentIndex {
        received: usize,
        expected_length: usize,
    },
    /// Occurrence numbers remain local classifier observations, never published names.
    CurrentDescriptorMismatch {
        occurrence: u64,
    },
    MissingConstruction {
        occurrence: u64,
    },
    UnknownParameterBinding,
    Exact(ResidualError),
}
