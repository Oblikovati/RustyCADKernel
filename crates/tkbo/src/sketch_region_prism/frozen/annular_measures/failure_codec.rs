// SPDX-License-Identifier: GPL-2.0-only
//! Bounded explicit-tag codec for an owned annular measure refusal. Version, count, operand,
//! work-overflow and cancellation causes are encoded with every partial certified measure; nested
//! direction/side-policy/cap/sweep/side/total numerical errors have no codec yet (`None`).
use super::super::proof_join::{encode_joined_block, encode_joined_text};
use super::replay::{measured_face, options, sweep};
use super::*;
use crate::sketch_region_prism::owned_request::names::encode_frozen_use_key;

/// Encode one refusal, or `None` when its nested numerical family has no explicit codec.
/// Example: `annular_measure_failure_bytes(&failure)` before moving the failure out.
pub(in crate::sketch_region_prism) fn annular_measure_failure_bytes(
    value: &AnnularMeasureFailure,
) -> Option<Vec<u8>> {
    let mut bytes = b"Martensite.AnnularMeasureRefusal/v1\0".to_vec();
    bytes.extend(value.operation.0.to_le_bytes());
    bytes.push(phase(value.phase));
    bytes.push(u8::from(value.entity.is_some()));
    if let Some(key) = &value.entity {
        encode_frozen_use_key(&mut bytes, key);
    }
    options(value.requested, &mut bytes);
    encode_joined_block(&value.binding, &mut bytes);
    partial(&value.partial, &mut bytes);
    cause(&value.cause, &mut bytes)?;
    Some(bytes)
}
fn phase(value: AnnularMeasurePhase) -> u8 {
    match value {
        AnnularMeasurePhase::Admission => 0,
        AnnularMeasurePhase::Caps => 1,
        AnnularMeasurePhase::Sweep => 2,
        AnnularMeasurePhase::Sides => 3,
        AnnularMeasurePhase::Total => 4,
        AnnularMeasurePhase::Commit => 5,
    }
}
fn partial(value: &AnnularMeasurePartial, bytes: &mut Vec<u8>) {
    bytes.extend((value.faces.len() as u64).to_le_bytes());
    for face in &value.faces {
        measured_face(face, bytes);
    }
    bytes.push(u8::from(value.signed_sweep.is_some()));
    if let Some(report) = &value.signed_sweep {
        sweep(report, bytes);
    }
    bytes.push(u8::from(value.total.is_some()));
    if let Some(report) = &value.total {
        encode_joined_total(report, bytes);
    }
    bytes.extend(value.side_subdivisions.to_le_bytes());
}
fn cause(value: &AnnularMeasureCause, bytes: &mut Vec<u8>) -> Option<()> {
    use AnnularMeasureCause as C;
    match value {
        C::Version { actual, expected } => {
            bytes.push(0);
            bytes.extend([*actual, *expected].map(u32::to_le_bytes).concat());
        }
        C::Count {
            quantity,
            actual,
            expected,
        } => {
            bytes.push(1);
            encode_joined_text(quantity, bytes);
            bytes.extend(
                [*actual as u64, *expected as u64]
                    .map(u64::to_le_bytes)
                    .concat(),
            );
        }
        C::Operand { required } => {
            bytes.push(2);
            encode_joined_text(required, bytes);
        }
        C::WorkOverflow {
            completed,
            additional,
        } => {
            bytes.push(3);
            bytes.extend([*completed, *additional].map(u32::to_le_bytes).concat());
        }
        C::Cancelled => bytes.push(4),
        _ => return None,
    }
    Some(())
}
