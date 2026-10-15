// SPDX-License-Identifier: GPL-2.0-only
//! Independent stable tags preserve class provenance and explicit unsupported families.
use super::*;
use crate::sketch_region_prism::{
    frozen::FreezePhase, region_certificate::embedding::PrismEmbeddingPhase,
};
pub(in crate::sketch_region_prism::diagnostic_envelope::publication) fn class(
    value: PrismPublicationClass,
) -> u8 {
    let class = match value {
        PrismPublicationClass::Failed => PrismDiagnosticClass::Failed,
        PrismPublicationClass::Incomplete => PrismDiagnosticClass::Incomplete,
        PrismPublicationClass::Cancelled => PrismDiagnosticClass::Cancelled,
    };
    shared::diagnostic_class_tag(class)
}
pub(in crate::sketch_region_prism::diagnostic_envelope::publication) fn pending(
    value: PublicationPendingPayload,
) -> u8 {
    match value {
        PublicationPendingPayload::JoinClosureQuery => 0,
        PublicationPendingPayload::JoinClosureRegion => 1,
        PublicationPendingPayload::JoinEmbedding => 2,
        PublicationPendingPayload::JoinCapture => 3,
        PublicationPendingPayload::JoinBinding => 4,
        PublicationPendingPayload::JoinLink => 5,
        PublicationPendingPayload::JoinIdentity => 6,
        PublicationPendingPayload::Validation => 7,
        PublicationPendingPayload::History => 8,
        PublicationPendingPayload::Measures => 9,
    }
}
pub(in crate::sketch_region_prism::diagnostic_envelope::publication) fn history(
    value: PrismHistoryPhase,
) -> u8 {
    match value {
        PrismHistoryPhase::Admission => 0,
        PrismHistoryPhase::Capture => 1,
        PrismHistoryPhase::Commit => 2,
    }
}
pub(super) fn join(value: PrismProofJoinPhase) -> u8 {
    match value {
        PrismProofJoinPhase::Admission => 0,
        PrismProofJoinPhase::Embedding => 1,
        PrismProofJoinPhase::Capture => 2,
        PrismProofJoinPhase::Link => 3,
        PrismProofJoinPhase::Commit => 4,
    }
}
pub(in crate::sketch_region_prism::diagnostic_envelope::publication) fn freeze(
    value: FreezePhase,
) -> u8 {
    match value {
        FreezePhase::Admission => 0,
        FreezePhase::Identity => 1,
        FreezePhase::Geometry => 2,
        FreezePhase::Incidence => 3,
        FreezePhase::Evidence => 4,
        FreezePhase::Commit => 5,
    }
}
pub(super) fn embedding(value: PrismEmbeddingPhase) -> u8 {
    match value {
        PrismEmbeddingPhase::Admission => 0,
        PrismEmbeddingPhase::CurrentBinding => 1,
        PrismEmbeddingPhase::Regularity => 2,
        PrismEmbeddingPhase::Incidences => 3,
        PrismEmbeddingPhase::FinalBinding => 4,
    }
}
