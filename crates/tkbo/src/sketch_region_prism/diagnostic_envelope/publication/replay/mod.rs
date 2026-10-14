// SPDX-License-Identifier: GPL-2.0-only
//! Explicit supported cause bytes and gaps; unsupported local owners are never Debug-serialized.
mod join;
mod tags;
use super::*;
use crate::sketch_region_prism::{
    diagnostic_envelope::replay as shared,
    frozen::proof_join::{encode_joined_block, encode_joined_text},
};
pub(super) use join::join_bytes;
impl OwnedPrismPublicationDiagnostic {
    /// Replay supported evidence and exact coverage; `diagnostic.replay_bytes()` excludes retained local owners.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"Martensite.OwnedPublicationDiagnostic/v1\0".to_vec();
        encode_joined_block(self.context.replay_bytes(), &mut bytes);
        bytes.extend(self.operation.0.to_le_bytes());
        bytes.extend([
            shared::publication_phase_tag(self.phase),
            tags::class(self.class),
        ]);
        bytes.push(u8::from(!self.gaps.is_empty()));
        cause(&self.cause, &mut bytes);
        bytes.extend((self.gaps.len() as u64).to_le_bytes());
        for value in &self.gaps {
            gap(value, &mut bytes);
        }
        bytes
    }
}
fn cause(value: &OwnedPublicationDiagnosticCause, bytes: &mut Vec<u8>) {
    match value {
        OwnedPublicationDiagnosticCause::UnsupportedVersion { actual, expected } => {
            bytes.push(0);
            versions(*actual, *expected, bytes);
        }
        OwnedPublicationDiagnosticCause::Stage(value) => {
            bytes.push(1);
            stage(value, bytes);
        }
        OwnedPublicationDiagnosticCause::HistoryCancelled { operation, phase } => {
            bytes.push(2);
            bytes.extend(operation.0.to_le_bytes());
            bytes.push(tags::history(*phase));
        }
        OwnedPublicationDiagnosticCause::Unprojected(value) => {
            bytes.extend([3, tags::pending(*value)])
        }
    }
}
fn stage(value: &OwnedPublicationStageDiagnostic, bytes: &mut Vec<u8>) {
    bytes.push(shared::publication_phase_tag(value.phase));
    bytes.push(tags::class(value.class));
    stage_cause(&value.cause, bytes);
}
fn stage_cause(value: &OwnedPublicationStageCause, bytes: &mut Vec<u8>) {
    use OwnedPublicationStageCause as C;
    match value {
        C::Join {
            encoded,
            class_origin,
            ..
        } => {
            joined_block(encoded, *class_origin, bytes);
        }
        C::Binding { proof } => {
            bytes.push(1);
            encode_joined_text(proof, bytes);
        }
        C::CallbackV1 { stop, downstream } => callback_v1(stop, downstream.as_deref(), bytes),
        C::CallbackV2 { stop, downstream } => callback_v2(stop, downstream.as_deref(), bytes),
        C::Unprojected(value) => bytes.extend([4, tags::pending(*value)]),
    }
}
fn callback_v1(
    stop: &OwnedPublicationCallback,
    downstream: Option<&OwnedPublicationStageCause>,
    bytes: &mut Vec<u8>,
) {
    bytes.push(2);
    callback(stop, bytes);
    bytes.push(u8::from(downstream.is_some()));
    if let Some(value) = downstream {
        stage_cause(value, bytes);
    }
}
fn callback_v2(
    stop: &OwnedPublicationCallback,
    downstream: Option<&OwnedPublicationStageDiagnostic>,
    bytes: &mut Vec<u8>,
) {
    bytes.push(3);
    callback(stop, bytes);
    bytes.push(u8::from(downstream.is_some()));
    if let Some(value) = downstream {
        stage(value, bytes);
    }
}
fn callback(value: &OwnedPublicationCallback, bytes: &mut Vec<u8>) {
    bytes.push(shared::publication_phase_tag(value.checkpoint.phase));
    bytes.extend((value.checkpoint.ordinal as u64).to_le_bytes());
    shared::append_callback_cause(&value.cause, bytes);
}
fn origin(value: PublicationClassOrigin, bytes: &mut Vec<u8>) {
    match value {
        PublicationClassOrigin::PublicationOperation => bytes.push(0),
        PublicationClassOrigin::RetainedStage(phase) => {
            bytes.extend([1, shared::publication_phase_tag(phase)])
        }
    }
}
fn gap(value: &PublicationDiagnosticGap, bytes: &mut Vec<u8>) {
    match value {
        PublicationDiagnosticGap::CallerDictionary => bytes.push(0),
        PublicationDiagnosticGap::RequestedPolicy => bytes.push(1),
        PublicationDiagnosticGap::Operation { actual, expected } => {
            bytes.push(2);
            bytes.extend(actual.0.to_le_bytes());
            bytes.extend(expected.0.to_le_bytes());
        }
        PublicationDiagnosticGap::UnretainedChildHeader => bytes.push(3),
        PublicationDiagnosticGap::Payload(value) => bytes.extend([4, tags::pending(*value)]),
        PublicationDiagnosticGap::UnprojectedEntity(value) => {
            bytes.push(5);
            encode_frozen_member_key(bytes, *value);
        }
    }
}
fn versions(actual: u32, expected: u32, bytes: &mut Vec<u8>) {
    bytes.extend(actual.to_le_bytes());
    bytes.extend(expected.to_le_bytes());
}
fn joined_block(encoded: &[u8], class_origin: PublicationClassOrigin, bytes: &mut Vec<u8>) {
    bytes.push(0);
    origin(class_origin, bytes);
    encode_joined_block(encoded, bytes);
}

#[cfg(test)]
mod tests;
