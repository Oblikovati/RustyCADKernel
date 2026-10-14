// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismProofJoinPhase {
    Admission,
    Embedding,
    Capture,
    Link,
    Commit,
}
// Legacy refusal payloads remain operation-local; they are not immutable successful body results.
pub(crate) struct PrismProofJoinFailure {
    pub operation: AuthoredPrismId,
    pub phase: PrismProofJoinPhase,
    pub entity: Option<FrozenPrismKey>,
    pub cause: PrismProofJoinCause,
}
pub(crate) enum PrismProofJoinCause {
    MissingClosureQuery,
    ClosureQuery(Box<crate::sketch_region_source_conformance::SourceConformanceCause>),
    ClosureRegion(Box<crate::sketch_region_prism::region_certificate::RegionCertificateCause>),
    Version { actual: u32, expected: u32 },
    Embedding(Box<PrismEmbeddingFailure>),
    IncompleteCapture(Box<PrismFreezeFailure>),
    FailedCapture(Box<PrismFreezeFailure>),
    Binding(Box<PrismBindingFailure>),
    Link(Box<PrismFreezeCause>),
    Cancelled { freeze_phase: Option<FreezePhase> },
    Stale { representation: &'static str },
}
pub(super) fn failure(
    operation: AuthoredPrismId,
    phase: PrismProofJoinPhase,
    cause: PrismProofJoinCause,
) -> PrismProofJoinFailure {
    let entity = match &cause {
        PrismProofJoinCause::IncompleteCapture(failure)
        | PrismProofJoinCause::FailedCapture(failure) => failure.entity,
        PrismProofJoinCause::Link(cause) => cause.primary_entity(),
        _ => None,
    };
    PrismProofJoinFailure {
        operation,
        phase,
        entity,
        cause,
    }
}
