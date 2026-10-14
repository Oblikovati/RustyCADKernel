// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::diagnostic_envelope::{
    append_native_failure, append_preparation_phase, publication_phase_tag,
};
pub(super) fn append(value: Option<&CompletePrismCallbackStop>, bytes: &mut Vec<u8>) {
    bytes.push(u8::from(value.is_some()));
    let Some(value) = value else {
        return;
    };
    phase(value.checkpoint.phase, bytes);
    words(&[value.checkpoint.ordinal as u64], bytes);
    bytes.push(u8::from(value.checkpoint.child_ordinal.is_some()));
    if let Some(ordinal) = value.checkpoint.child_ordinal {
        words(&[ordinal as u64], bytes);
    }
    match &value.cause {
        CompletePrismCallbackCause::Cancelled => bytes.push(0),
        CompletePrismCallbackCause::Failed(value) => {
            bytes.push(1);
            append_native_failure(value, bytes);
        }
        CompletePrismCallbackCause::Exhausted => bytes.push(2),
    }
}
fn phase(value: CompletePrismPhase, bytes: &mut Vec<u8>) {
    match value {
        CompletePrismPhase::Preparation(value) => {
            bytes.push(0);
            append_preparation_phase(value, bytes);
        }
        CompletePrismPhase::Resolution => bytes.push(1),
        CompletePrismPhase::Publication(value) => bytes.extend([2, publication_phase_tag(value)]),
        CompletePrismPhase::Names => bytes.push(3),
        CompletePrismPhase::Reservations => bytes.push(4),
        CompletePrismPhase::Body => bytes.push(5),
        CompletePrismPhase::Volume => bytes.push(6),
        CompletePrismPhase::Commit => bytes.push(7),
    }
}
