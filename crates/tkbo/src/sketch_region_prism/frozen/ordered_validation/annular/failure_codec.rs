// SPDX-License-Identifier: GPL-2.0-only
//! Bounded explicit-tag codec for an owned annular validation refusal. Version, binding,
//! cancellation and callback causes are encoded with every retained partial certificate; nested
//! topology/orientation/conformance/intersection/tolerance failures have no codec yet (`None`).
use super::replay::{topology as topology_bytes, trace};
use super::*;
use crate::sketch_region_prism::diagnostic_envelope::append_native_failure;
use crate::sketch_region_prism::exact_tolerance::encode_stored_tolerance_cap;
use crate::sketch_region_prism::frozen::proof_join::{encode_joined_block, encode_joined_text};

/// Encode one refusal, or `None` when a nested family has no explicit codec.
/// Example: `annular_validation_failure_bytes(&failure)` before moving the failure out.
pub(in crate::sketch_region_prism) fn annular_validation_failure_bytes(
    value: &AnnularValidationFailure,
) -> Option<Vec<u8>> {
    let mut bytes = b"Martensite.AnnularValidationRefusal/v1\0".to_vec();
    bytes.extend(value.operation.0.to_le_bytes());
    bytes.extend(value.requested.version.to_le_bytes());
    encode_stored_tolerance_cap(value.requested.stored_cap, &mut bytes);
    checkpoint(value.checkpoint, &mut bytes);
    bytes.push(class(value.class));
    trace(&value.trace, &mut bytes);
    cause(&value.cause, &mut bytes)?;
    encode_joined_block(&value.source, &mut bytes);
    partial(&value.partial, &mut bytes);
    Some(bytes)
}
fn class(value: OrderedPrismFailureClass) -> u8 {
    match value {
        OrderedPrismFailureClass::Failed => 0,
        OrderedPrismFailureClass::Incomplete => 1,
        OrderedPrismFailureClass::Cancelled => 2,
    }
}
fn checkpoint(value: AnnularValidationCheckpoint, bytes: &mut Vec<u8>) {
    let stage = OrderedPrismStage::ALL
        .iter()
        .position(|s| *s == value.stage);
    bytes.push(stage.expect("ordered stage") as u8);
    check(value.check, bytes);
    bytes.extend((value.ordinal as u64).to_le_bytes());
}
fn check(value: AnnularValidationCheck, bytes: &mut Vec<u8>) {
    // Nested fieldless phases use their declaration discriminants; a reorder is a format change.
    use AnnularValidationCheck as C;
    match value {
        C::Admission => bytes.push(0),
        C::Topology(point) => {
            bytes.extend([1, point.invariant as u8]);
            bytes.extend((point.completed as u64).to_le_bytes());
        }
        C::Orientation { commit } => bytes.extend([2, u8::from(commit)]),
        C::Conformance(phase) => bytes.extend([3, phase as u8]),
        C::Incidence { completed } => {
            bytes.push(4);
            bytes.extend((completed as u64).to_le_bytes());
        }
        C::Tolerance(phase) => bytes.extend([5, phase as u8]),
        C::Commit => bytes.push(6),
    }
}
fn cause(value: &AnnularValidationCause, bytes: &mut Vec<u8>) -> Option<()> {
    use AnnularValidationCause as C;
    match value {
        C::Version { actual, expected } => {
            bytes.push(0);
            bytes.extend([*actual, *expected].map(u32::to_le_bytes).concat());
        }
        C::Binding { proof } => {
            bytes.push(1);
            encode_joined_text(proof, bytes);
        }
        C::Cancelled => bytes.push(2),
        C::Callback { stop, downstream } => callback(stop, downstream.as_deref(), bytes)?,
        _ => return None,
    }
    Some(())
}
fn callback(
    stop: &AnnularValidationCallbackStop,
    downstream: Option<&ValidationStop>,
    bytes: &mut Vec<u8>,
) -> Option<()> {
    bytes.push(3);
    checkpoint(stop.checkpoint, bytes);
    match &stop.cause {
        AnnularValidationCallbackCause::Cancelled => bytes.push(0),
        AnnularValidationCallbackCause::Failed(failure) => {
            bytes.push(1);
            append_native_failure(failure, bytes);
        }
        AnnularValidationCallbackCause::Exhausted => bytes.push(2),
    }
    bytes.push(u8::from(downstream.is_some()));
    let Some(child) = downstream else {
        return Some(());
    };
    bytes.push(class(child.class));
    cause(&child.cause, bytes)
}
fn partial(value: &AnnularValidationPartial, bytes: &mut Vec<u8>) {
    let proofs = [
        value.topology.as_deref().map(topology_bytes),
        value.orientation.as_ref().map(|p| p.replay_bytes()),
        value.conformance.as_ref().map(|p| p.replay_bytes()),
        value.intersection.as_ref().map(|p| p.replay_bytes()),
        value.tolerance.as_ref().map(|p| p.replay_bytes()),
    ];
    for proof in proofs {
        bytes.push(u8::from(proof.is_some()));
        if let Some(proof) = proof {
            encode_joined_block(&proof, bytes);
        }
    }
}
