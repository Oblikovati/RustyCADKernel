// SPDX-License-Identifier: GPL-2.0-only
//! A bounded codec admits exact scalar/native join causes; other families stay explicitly incomplete.
use super::*;
pub(in crate::sketch_region_prism::diagnostic_envelope::publication) fn join_bytes(
    value: &OwnedPrismJoinRefusal,
) -> Result<Vec<u8>, PublicationPendingPayload> {
    let mut bytes = b"Martensite.PublicationJoinDiagnostic/v1\0".to_vec();
    bytes.extend(value.operation().0.to_le_bytes());
    bytes.push(tags::join(value.phase()));
    bytes.push(match value.class() {
        PrismRefusalClass::Failed => 2,
        PrismRefusalClass::Incomplete => 1,
        PrismRefusalClass::Cancelled => 3,
    });
    bytes.push(u8::from(value.entity().is_some()));
    if let Some(key) = value.entity() {
        encode_frozen_member_key(&mut bytes, key);
    }
    cause(value.cause(), &mut bytes)?;
    Ok(bytes)
}
fn cause(
    value: &OwnedPrismJoinCause,
    bytes: &mut Vec<u8>,
) -> Result<(), PublicationPendingPayload> {
    use OwnedPrismJoinCause as C;
    match value {
        C::MissingClosureQuery => bytes.push(0),
        C::Version { actual, expected } => {
            bytes.push(1);
            versions(*actual, *expected, bytes);
        }
        C::Cancelled { freeze_phase } => cancellation(*freeze_phase, bytes),
        C::Stale { representation } => {
            bytes.push(3);
            encode_joined_text(representation, bytes);
        }
        other => composite(other, bytes)?,
    }
    Ok(())
}
fn cancellation(
    phase: Option<crate::sketch_region_prism::frozen::FreezePhase>,
    bytes: &mut Vec<u8>,
) {
    bytes.extend([2, u8::from(phase.is_some())]);
    if let Some(phase) = phase {
        bytes.push(tags::freeze(phase));
    }
}
fn composite(
    value: &OwnedPrismJoinCause,
    bytes: &mut Vec<u8>,
) -> Result<(), PublicationPendingPayload> {
    match value {
        OwnedPrismJoinCause::Binding(value) => {
            bytes.push(4);
            binding(value, bytes)?;
        }
        OwnedPrismJoinCause::Embedding(value) => {
            bytes.push(5);
            embedding(value, bytes)?;
        }
        _ => return Err(PublicationPendingPayload::JoinIdentity),
    }
    Ok(())
}
fn binding(
    value: &OwnedPrismBindingCause,
    bytes: &mut Vec<u8>,
) -> Result<(), PublicationPendingPayload> {
    match value {
        OwnedPrismBindingCause::Cancelled => bytes.push(0),
        OwnedPrismBindingCause::Region(OwnedPrismRegionCause::Mapped(value)) => {
            bytes.push(1);
            shared::append_native_failure(value, bytes);
        }
        _ => return Err(PublicationPendingPayload::JoinBinding),
    }
    Ok(())
}
fn embedding(
    value: &OwnedPrismEmbeddingFailure,
    bytes: &mut Vec<u8>,
) -> Result<(), PublicationPendingPayload> {
    if value.original.is_some() {
        return Err(PublicationPendingPayload::JoinIdentity);
    }
    encode_joined_embedding_options(value.requested, bytes);
    bytes.push(tags::embedding(value.phase));
    embedding_cause(&value.cause, bytes)
}
fn embedding_cause(
    value: &OwnedPrismEmbeddingCause,
    bytes: &mut Vec<u8>,
) -> Result<(), PublicationPendingPayload> {
    match value {
        OwnedPrismEmbeddingCause::Version { actual, expected } => {
            bytes.push(0);
            versions(*actual, *expected, bytes);
        }
        OwnedPrismEmbeddingCause::Binding(value) => {
            bytes.push(1);
            binding(value, bytes)?;
        }
        _ => return Err(PublicationPendingPayload::JoinEmbedding),
    }
    Ok(())
}
