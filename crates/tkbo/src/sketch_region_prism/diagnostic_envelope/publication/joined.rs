// SPDX-License-Identifier: GPL-2.0-only
//! Clone only supported scalar/native causes, never working transaction owners; retain originals on any gap.
use super::*;
use crate::sketch_region_prism::{
    frozen::PrismPublicationBindings,
    region_certificate::{
        RegionCertificateCause, current_binding::PrismBindingFailure, embedding::*,
    },
};
pub(super) fn project(
    value: &PrismProofJoinFailure,
    class: PrismPublicationClass,
    bindings: &PrismPublicationBindings,
) -> Result<(Box<OwnedPrismJoinRefusal>, Vec<u8>), PublicationPendingPayload> {
    let cause = copy_cause(&value.cause).ok_or_else(|| family(&value.cause))?;
    let copied = copied_failure(value, cause);
    let context = PrismRefusalIdentityContext { bindings };
    let PrismJoinRefusalProjection::Projected(evidence) =
        project_join_refusal(copied, refusal_class(class), &context)
    else {
        return Err(PublicationPendingPayload::JoinIdentity);
    };
    let encoded = replay::join_bytes(&evidence)?;
    Ok((evidence, encoded))
}
fn refusal_class(value: PrismPublicationClass) -> PrismRefusalClass {
    match value {
        PrismPublicationClass::Failed => PrismRefusalClass::Failed,
        PrismPublicationClass::Incomplete => PrismRefusalClass::Incomplete,
        PrismPublicationClass::Cancelled => PrismRefusalClass::Cancelled,
    }
}
fn copy_cause(value: &PrismProofJoinCause) -> Option<PrismProofJoinCause> {
    use PrismProofJoinCause as C;
    match value {
        C::MissingClosureQuery => Some(C::MissingClosureQuery),
        C::Version { actual, expected } => Some(C::Version {
            actual: *actual,
            expected: *expected,
        }),
        C::Cancelled { freeze_phase } => Some(C::Cancelled {
            freeze_phase: *freeze_phase,
        }),
        C::Stale { representation } => Some(C::Stale { representation }),
        C::Binding(value) => copy_binding(value).map(|value| C::Binding(Box::new(value))),
        C::Embedding(value) => copy_embedding(value).map(|value| C::Embedding(Box::new(value))),
        _ => None,
    }
}
fn copy_binding(value: &PrismBindingFailure) -> Option<PrismBindingFailure> {
    match value {
        PrismBindingFailure::Cancelled => Some(PrismBindingFailure::Cancelled),
        PrismBindingFailure::Region(RegionCertificateCause::Mapped(value)) => Some(
            PrismBindingFailure::Region(RegionCertificateCause::Mapped(value.clone())),
        ),
        _ => None,
    }
}
fn copy_embedding(value: &PrismEmbeddingFailure) -> Option<PrismEmbeddingFailure> {
    if value.original.is_some() {
        return None;
    }
    let cause = copy_embedding_cause(&value.cause)?;
    Some(PrismEmbeddingFailure {
        requested: value.requested,
        phase: value.phase,
        original: None,
        cause,
    })
}
pub(super) fn family(value: &PrismProofJoinCause) -> PublicationPendingPayload {
    match value {
        PrismProofJoinCause::ClosureQuery(_) => PublicationPendingPayload::JoinClosureQuery,
        PrismProofJoinCause::ClosureRegion(_) => PublicationPendingPayload::JoinClosureRegion,
        PrismProofJoinCause::Embedding(_) => PublicationPendingPayload::JoinEmbedding,
        PrismProofJoinCause::IncompleteCapture(_) | PrismProofJoinCause::FailedCapture(_) => {
            PublicationPendingPayload::JoinCapture
        }
        PrismProofJoinCause::Binding(_) => PublicationPendingPayload::JoinBinding,
        PrismProofJoinCause::Link(_) => PublicationPendingPayload::JoinLink,
        _ => PublicationPendingPayload::JoinIdentity,
    }
}
fn copied_failure(
    value: &PrismProofJoinFailure,
    cause: PrismProofJoinCause,
) -> PrismProofJoinFailure {
    PrismProofJoinFailure {
        operation: value.operation,
        phase: value.phase,
        entity: value.entity,
        cause,
    }
}
fn copy_embedding_cause(value: &PrismEmbeddingCause) -> Option<PrismEmbeddingCause> {
    let cause = match value {
        PrismEmbeddingCause::Version { actual, expected } => PrismEmbeddingCause::Version {
            actual: *actual,
            expected: *expected,
        },
        PrismEmbeddingCause::Binding(value) => {
            PrismEmbeddingCause::Binding(Box::new(copy_binding(value)?))
        }
        _ => return None,
    };
    Some(cause)
}
