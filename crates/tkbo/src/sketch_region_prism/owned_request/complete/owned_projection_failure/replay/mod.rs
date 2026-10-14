// SPDX-License-Identifier: GPL-2.0-only
//! Full retained projection evidence: no Debug fallback, invented child views or live reconstruction.
mod body;
mod callback;
mod names;
mod reservations;
mod volume;
use super::*;
use crate::sketch_region_prism::{
    frozen::proof_join::{encode_joined_block, encode_joined_text},
    owned_request::{names::*, request_replay::encode_owned_prism_request},
};
impl OwnedProjectionRefusal {
    /// Encode full retained owner, precise typed cause and callback; `failure.replay_bytes()` is independent of live input.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        let mut bytes = b"Martensite.OwnedProjectionRefusal/v1\0".to_vec();
        encode_joined_block(&encode_owned_prism_request(self.request()), &mut bytes);
        let context = SemanticReplayEnvelope::new(
            self.plan().clone(),
            self.accepted_publication_context().replay_bytes(),
        );
        encode_joined_block(context.replay_bytes(), &mut bytes);
        bytes.push(0); // Retained coverage: accepted publication only, no dropped pure projections.
        classification(self, &mut bytes);
        callback::append(self.callback(), &mut bytes);
        cause(self.cause(), &mut bytes);
        bytes
    }
}
fn cause(value: &CompleteProjectionCause, bytes: &mut Vec<u8>) {
    bytes.push(cause_tag(value));
    match value {
        CompleteProjectionCause::Callback => (),
        CompleteProjectionCause::Names(value) => names::append(value, bytes),
        CompleteProjectionCause::Reservations(value) => reservations::append(value, bytes),
        CompleteProjectionCause::Body(value) => body::append(value, bytes),
        CompleteProjectionCause::Volume(value) => volume::append(value, bytes),
    }
}
fn cause_tag(value: &CompleteProjectionCause) -> u8 {
    match value {
        CompleteProjectionCause::Callback => 0,
        CompleteProjectionCause::Names(_) => 1,
        CompleteProjectionCause::Reservations(_) => 2,
        CompleteProjectionCause::Body(_) => 3,
        CompleteProjectionCause::Volume(_) => 4,
    }
}
fn classification(value: &OwnedProjectionRefusal, bytes: &mut Vec<u8>) {
    let (class, origin) = value.classification();
    bytes.extend([class_tag(class), origin_tag(origin)]);
    match value.retained_child_class() {
        RetainedProjectionClass::Unavailable => bytes.push(0),
        RetainedProjectionClass::Body(value) => {
            bytes.push(1);
            body::class(value, bytes);
        }
        RetainedProjectionClass::Volume(value) => {
            bytes.push(2);
            volume::class(value, bytes);
        }
    }
}
fn class_tag(value: ProjectionRefusalClass) -> u8 {
    match value {
        ProjectionRefusalClass::Failed => 0,
        ProjectionRefusalClass::Cancelled => 1,
        ProjectionRefusalClass::Incomplete => 2,
    }
}
fn origin_tag(value: ProjectionClassOrigin) -> u8 {
    match value {
        ProjectionClassOrigin::OuterCallback => 0,
        ProjectionClassOrigin::RetainedChild => 1,
        ProjectionClassOrigin::BridgePolicyV1 => 2,
    }
}
fn words(values: &[u64], bytes: &mut Vec<u8>) {
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}
fn slot(value: SemanticNameSlot, bytes: &mut Vec<u8>) {
    let tags = [value.stage as u8, value.role as u8];
    bytes.extend_from_slice(&tags);
}
fn optional_slot(value: Option<SemanticNameSlot>, bytes: &mut Vec<u8>) {
    bytes.push(u8::from(value.is_some()));
    if let Some(value) = value {
        slot(value, bytes);
    }
}

#[cfg(test)]
mod scalar_tests;
#[cfg(test)]
mod tests;

fn versions(actual: u32, expected: u32, bytes: &mut Vec<u8>) {
    bytes.extend(actual.to_le_bytes());
    bytes.extend(expected.to_le_bytes());
}
