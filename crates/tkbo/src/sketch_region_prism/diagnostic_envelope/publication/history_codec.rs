// SPDX-License-Identifier: GPL-2.0-only
//! Bounded explicit-tag codec for owned history findings; families holding local owners or
//! untagged nested payloads (legacy, lineage, frozen, geometry) stay explicitly unprojected.
use super::*;
use crate::sketch_region_prism::diagnostic_envelope::append_native_failure;
use crate::sketch_region_prism::frozen::FrozenOriginalKey;
use crate::sketch_region_prism::frozen::complete_history::*;
use crate::sketch_region_prism::frozen::proof_join::encode_joined_text;

type Codec = Result<(), PublicationPendingPayload>;
const PENDING: PublicationPendingPayload = PublicationPendingPayload::History;

/// Encode one owned finding, or report the history family as pending.
/// Example: `finding_bytes(&failure.finding)?` before moving the finding out of its carrier.
pub(super) fn finding_bytes(
    value: &PrismHistoryFinding,
) -> Result<Vec<u8>, PublicationPendingPayload> {
    let mut bytes = b"Martensite.PublicationHistoryDiagnostic/v1\0".to_vec();
    bytes.extend(value.operation.0.to_le_bytes());
    bytes.extend([phase(value.phase), class(value.class)]);
    bytes.push(u8::from(value.entity.is_some()));
    if let Some(key) = &value.entity {
        use_key(key, &mut bytes);
    }
    cause(&value.cause, &mut bytes)?;
    Ok(bytes)
}
fn phase(value: PrismHistoryPhase) -> u8 {
    match value {
        PrismHistoryPhase::Admission => 0,
        PrismHistoryPhase::Capture => 1,
        PrismHistoryPhase::Commit => 2,
    }
}
fn class(value: PrismHistoryFailureClass) -> u8 {
    match value {
        PrismHistoryFailureClass::Incomplete => 1,
        PrismHistoryFailureClass::Failed => 2,
    }
}
fn stage(value: PrismHistoryStage) -> u8 {
    match value {
        PrismHistoryStage::Original => 0,
        PrismHistoryStage::InputCopy => 1,
        PrismHistoryStage::Partition => 2,
        PrismHistoryStage::CandidateCopy => 3,
        PrismHistoryStage::Selected => 4,
        PrismHistoryStage::Complement => 5,
        PrismHistoryStage::SweepBase => 6,
    }
}
fn kind(value: HistoricalDiagnosticKind) -> u8 {
    match value {
        HistoricalDiagnosticKind::GeneratedObservation => 0,
        HistoricalDiagnosticKind::CandidateCompletion => 1,
        HistoricalDiagnosticKind::CandidateObservation => 2,
        HistoricalDiagnosticKind::PartitionRead => 3,
        HistoricalDiagnosticKind::ArrangementCompletion => 4,
    }
}
fn use_key(value: &PrismHistoryUseKey, bytes: &mut Vec<u8>) {
    bytes.extend((value.0.len() as u64).to_le_bytes());
    for (member, directed) in &value.0 {
        bytes.extend(member.operation.0.to_le_bytes());
        bytes.push(stage(member.stage));
        bytes.extend(member.member.0.to_le_bytes());
        bytes.push(member.kind);
        bytes.extend(directed.0.to_le_bytes());
    }
}
fn cause(value: &PrismHistoryCause, bytes: &mut Vec<u8>) -> Codec {
    use PrismHistoryCause as C;
    match value {
        C::AtUse { entity, cause: c } => {
            bytes.push(0);
            use_key(entity, bytes);
            cause(c, bytes)
        }
        C::Missing { required } => text(2, required, bytes),
        C::Changed { operand } => text(3, operand, bytes),
        other => structured(other, bytes),
    }
}
fn structured(value: &PrismHistoryCause, bytes: &mut Vec<u8>) -> Codec {
    use PrismHistoryCause as C;
    match value {
        C::Count {
            required: r,
            actual: a,
            expected: e,
        } => count(1, r, *a, *e, bytes),
        C::ParentsMismatch {
            stage: s,
            actual: a,
            expected: e,
        } => parents(*s, a, e, bytes),
        other => scalar(other, bytes),
    }
}
fn scalar(value: &PrismHistoryCause, bytes: &mut Vec<u8>) -> Codec {
    use PrismHistoryCause as C;
    match value {
        C::Bounds(value) => bounds(value, bytes),
        C::ContextRead { stage: s, cause } => {
            let repeated = matches!(cause, HistoricalContextReadCause::RepeatedAncestor);
            bytes.extend([6, stage(*s), u8::from(repeated)]);
            Ok(())
        }
        C::Native(value) => native(7, value, bytes),
        C::LineageCancelled(value) => {
            bytes.extend([8, super::replay::freeze_tag(*value)]);
            Ok(())
        }
        other => unsupported(other, bytes),
    }
}
fn unsupported(value: &PrismHistoryCause, bytes: &mut Vec<u8>) -> Codec {
    let PrismHistoryCause::UnsupportedDiagnostic {
        stage: s,
        kind: k,
        observation,
        retained_rows,
    } = value
    else {
        return Err(PENDING);
    };
    bytes.extend([9, stage(*s), kind(*k), u8::from(observation.is_some())]);
    bytes.extend((observation.unwrap_or(0) as u64).to_le_bytes());
    bytes.extend((*retained_rows as u64).to_le_bytes());
    Ok(())
}
fn native(tag: u8, value: &rustycad_tkernel::standard::Failure, bytes: &mut Vec<u8>) -> Codec {
    bytes.push(tag);
    append_native_failure(value, bytes);
    Ok(())
}
fn count(tag: u8, required: &str, actual: usize, expected: usize, bytes: &mut Vec<u8>) -> Codec {
    text(tag, required, bytes)?;
    bytes.extend((actual as u64).to_le_bytes());
    bytes.extend((expected as u64).to_le_bytes());
    Ok(())
}
fn text(tag: u8, value: &str, bytes: &mut Vec<u8>) -> Codec {
    bytes.push(tag);
    encode_joined_text(value, bytes);
    Ok(())
}
fn parents(
    value: PrismHistoryStage,
    actual: &[Vec<FrozenOriginalKey>; 3],
    expected: &[Vec<FrozenOriginalKey>; 3],
    bytes: &mut Vec<u8>,
) -> Codec {
    bytes.extend([4, stage(value)]);
    for class in actual.iter().chain(expected.iter()) {
        bytes.extend((class.len() as u64).to_le_bytes());
        for key in class {
            bytes.extend(key.source.to_le_bytes());
            bytes.extend(key.member.0.to_le_bytes());
            bytes.extend(key.directed_use.0.to_le_bytes());
        }
    }
    Ok(())
}
fn bounds(value: &HistoryBoundCause, bytes: &mut Vec<u8>) -> Codec {
    bytes.push(5);
    use HistoryBoundCause as B;
    match value {
        B::Setting {
            quantity: q,
            actual: a,
            minimum: n,
            maximum: m,
        } => {
            count(0, q, *a, *n, bytes)?;
            bytes.extend((*m as u64).to_le_bytes());
            Ok(())
        }
        other => bound_operands(other, bytes),
    }
}
fn bound_operands(value: &HistoryBoundCause, bytes: &mut Vec<u8>) -> Codec {
    use HistoryBoundCause as B;
    match value {
        B::Exhausted {
            quantity: q,
            actual: a,
            maximum: m,
        } => count(1, q, *a, *m, bytes),
        B::Setting { .. } => unreachable!("setting encoded by bounds"),
        B::AncestorCycle {
            ancestor_depth: a,
            repeated_depth: r,
        } => {
            bytes.push(2);
            bytes.extend([*a as u64, *r as u64].map(u64::to_le_bytes).concat());
            Ok(())
        }
        B::Native(value) => native(3, value, bytes),
    }
}

#[cfg(test)]
#[path = "history_codec_tests.rs"]
mod tests;
