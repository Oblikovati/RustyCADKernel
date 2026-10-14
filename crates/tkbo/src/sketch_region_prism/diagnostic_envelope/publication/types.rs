// SPDX-License-Identifier: GPL-2.0-only
//! Projection coverage is separate from the original outer and actual retained child classes.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PublicationPendingPayload {
    JoinClosureQuery,
    JoinClosureRegion,
    JoinEmbedding,
    JoinCapture,
    JoinBinding,
    JoinLink,
    JoinIdentity,
    Validation,
    History,
    Measures,
}
/// Names the actual retained header; the join error itself has no independently stored class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PublicationClassOrigin {
    PublicationOperation,
    RetainedStage(PrismPublicationPhase),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PublicationDiagnosticGap {
    CallerDictionary,
    RequestedPolicy,
    Operation {
        actual: AuthoredPrismId,
        expected: AuthoredPrismId,
    },
    UnretainedChildHeader,
    /// A complete authored observation, not a caller-qualified persistence identity.
    UnprojectedEntity(crate::sketch_region_prism::frozen::FrozenPrismKey),
    Payload(PublicationPendingPayload),
}
pub(crate) struct OwnedPublicationStageDiagnostic {
    pub phase: PrismPublicationPhase,
    pub class: PrismPublicationClass,
    pub cause: OwnedPublicationStageCause,
}
pub(crate) struct OwnedPublicationCallback {
    pub checkpoint: PrismPublicationCheckpoint,
    pub cause: OwnedDiagnosticCallbackCause,
}
pub(crate) enum OwnedPublicationStageCause {
    Join {
        evidence: Box<OwnedPrismJoinRefusal>,
        class_origin: PublicationClassOrigin,
        encoded: Vec<u8>,
    },
    Binding {
        proof: &'static str,
    },
    CallbackV1 {
        stop: OwnedPublicationCallback,
        downstream: Option<Box<Self>>,
    },
    CallbackV2 {
        stop: OwnedPublicationCallback,
        downstream: Option<Box<OwnedPublicationStageDiagnostic>>,
    },
    Unprojected(PublicationPendingPayload),
}
pub(crate) enum OwnedPublicationDiagnosticCause {
    UnsupportedVersion {
        actual: u32,
        expected: u32,
    },
    Stage(Box<OwnedPublicationStageDiagnostic>),
    HistoryCancelled {
        operation: AuthoredPrismId,
        phase: PrismHistoryPhase,
    },
    Unprojected(PublicationPendingPayload),
}
pub(crate) struct OwnedPrismPublicationDiagnostic {
    pub(super) context: OwnedPrismDiagnosticContext,
    pub(super) operation: AuthoredPrismId,
    pub(super) phase: PrismPublicationPhase,
    pub(super) class: PrismPublicationClass,
    pub(super) cause: OwnedPublicationDiagnosticCause,
    pub(super) gaps: Vec<PublicationDiagnosticGap>,
}
/// Only the owned diagnostic is publishable; this carrier retains unsupported local owners.
pub(crate) struct InvokedPublicationRefusalProjection<'a> {
    pub(super) diagnostic: Box<OwnedPrismPublicationDiagnostic>,
    pub(super) original: Option<Box<PrismPublicationFailure<'a>>>,
}
