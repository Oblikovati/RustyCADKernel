// SPDX-License-Identifier: GPL-2.0-only
//! Owned diagnostic values exclude all live preparation owners and original process identities.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismDiagnosticClass {
    Declined,
    Incomplete,
    Failed,
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismDiagnosticPhase {
    PreparationBoundary(OwnedPrismPhase),
    Publication(super::super::publication::PrismPublicationPhase),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiagnosticProjectionCoverage {
    Complete,
    Incomplete,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UnprojectedPreparationPayload {
    Census,
    ContactTransport,
    RegionPrism,
    StageCompletion,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PrismDiagnosticGap {
    PreparationCause(UnprojectedPreparationPayload),
    OuterOwners(Vec<RetainedPreparationStage>),
    MissingCallbackEvidence,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum OwnedPreparationDiagnosticCause {
    Admission(OwnedPrismAdmissionCause),
    Native(Failure),
    Callback,
    StageUnavailable { class: OwnedPreparationClass },
    MissingLocalState { expected: &'static str },
    Unprojected(UnprojectedPreparationPayload),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum OwnedDiagnosticCallbackCause {
    Cancelled,
    Failed(Failure),
    Exhausted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OwnedPreparationCallback {
    pub checkpoint: OwnedPrismCheckpoint,
    pub cause: OwnedDiagnosticCallbackCause,
}
pub(crate) struct OwnedPrismDiagnosticContext {
    pub(super) request: OwnedPolynomialPrismRequest,
    pub(super) plan: Option<SingleBoundaryNamePlan>,
    pub(super) replay: Vec<u8>,
}
pub(crate) struct OwnedPrismDiagnostic {
    pub(super) context: OwnedPrismDiagnosticContext,
    pub(super) phase: PrismDiagnosticPhase,
    pub(super) class: PrismDiagnosticClass,
    pub(super) cause: OwnedPreparationDiagnosticCause,
    pub(super) callback: Option<OwnedPreparationCallback>,
    pub(super) gaps: Vec<PrismDiagnosticGap>,
}
/// This operation-local carrier retains unsupported owners; only its diagnostic is immutable publication evidence.
pub(crate) enum PreparationRefusalProjection {
    Projected(Box<OwnedPrismDiagnostic>),
    Incomplete {
        diagnostic: Box<OwnedPrismDiagnostic>,
        original: Box<OwnedPrismPreparationFailure>,
    },
}
