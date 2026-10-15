// SPDX-License-Identifier: GPL-2.0-only
//! Owned annular join refusals: identity, version, name-budget, native callback and cancellation
//! causes own their operands. Borrowed embedding/context/original-region families and frozen
//! capture/link payloads remain pending payloads.
use super::super::replay::freeze_tag;
use super::*;
use crate::sketch_region_prism::frozen::{FreezePhase, annular_join::AnnularFrozenPhase};
use rustycad_tkernel::standard::Failure;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum OwnedAnnularJoinCause {
    AuthoredIdentity {
        quantity: &'static str,
        actual: AuthoredPrismId,
        expected: AuthoredPrismId,
    },
    Version {
        actual: u32,
        expected: u32,
    },
    NameBudget {
        maximum: usize,
        measured: Option<usize>,
    },
    Callback(Failure),
    Cancelled {
        freeze_phase: Option<FreezePhase>,
    },
}
/// Owned join header and cause with explicit-tag bytes; `join.cause()`.
pub(in crate::sketch_region_prism) struct OwnedAnnularJoinRefusal {
    pub(super) operation: AuthoredPrismId,
    pub(super) phase: AnnularFrozenPhase,
    pub(super) requested: AnnularFrozenOptions,
    pub(super) cause: OwnedAnnularJoinCause,
    pub(super) encoded: Vec<u8>,
}
impl OwnedAnnularJoinRefusal {
    /// Read the copied typed cause; `join.cause()`.
    pub(in crate::sketch_region_prism) fn cause(&self) -> &OwnedAnnularJoinCause {
        &self.cause
    }
    /// Read the explicit-tag bytes; `join.encoded()`.
    pub(in crate::sketch_region_prism) fn encoded(&self) -> &[u8] {
        &self.encoded
    }
}
/// Copy an owned-operand join refusal, or name the pending family.
/// Example: `own_join(&failure)?` before the outer projection discards the borrowed carrier.
pub(super) fn own_join(
    failure: &AnnularFrozenFailure<'_>,
) -> Result<OwnedAnnularJoinRefusal, PublicationPendingPayload> {
    let cause = owned_cause(&failure.cause)?;
    let mut refusal = OwnedAnnularJoinRefusal {
        operation: failure.operation,
        phase: failure.phase,
        requested: failure.requested,
        cause,
        encoded: Vec::new(),
    };
    refusal.encoded = encode(&refusal);
    Ok(refusal)
}
fn owned_cause(
    value: &AnnularFrozenCause<'_>,
) -> Result<OwnedAnnularJoinCause, PublicationPendingPayload> {
    use AnnularFrozenCause as C;
    use OwnedAnnularJoinCause as O;
    use PublicationPendingPayload as P;
    Ok(match value {
        C::AuthoredIdentity {
            quantity,
            actual,
            expected,
        } => O::AuthoredIdentity {
            quantity,
            actual: *actual,
            expected: *expected,
        },
        C::Version { actual, expected } => O::Version {
            actual: *actual,
            expected: *expected,
        },
        C::NameBudget { maximum, measured } => O::NameBudget {
            maximum: *maximum,
            measured: *measured,
        },
        C::Callback(failure) => O::Callback(failure.clone()),
        C::Cancelled { freeze_phase } => O::Cancelled {
            freeze_phase: *freeze_phase,
        },
        C::Embedding(_) => return Err(P::JoinEmbedding),
        C::Context(_) => return Err(P::JoinBinding),
        C::Originals(_) => return Err(P::JoinClosureRegion),
        C::IncompleteCapture(_) | C::FailedCapture(_) => return Err(P::JoinCapture),
        C::Link(_) => return Err(P::JoinLink),
    })
}
fn encode(value: &OwnedAnnularJoinRefusal) -> Vec<u8> {
    let mut bytes = b"Martensite.AnnularJoinRefusal/v1\0".to_vec();
    bytes.extend(value.operation.0.to_le_bytes());
    bytes.push(phase(value.phase));
    bytes.extend(value.requested.version.to_le_bytes());
    bytes.extend((value.requested.maximum_name_units as u64).to_le_bytes());
    cause(&value.cause, &mut bytes);
    bytes
}
fn phase(value: AnnularFrozenPhase) -> u8 {
    match value {
        AnnularFrozenPhase::Admission => 0,
        AnnularFrozenPhase::CurrentBinding => 1,
        AnnularFrozenPhase::Capture => 2,
        AnnularFrozenPhase::Link => 3,
        AnnularFrozenPhase::Commit => 4,
    }
}
fn cause(value: &OwnedAnnularJoinCause, bytes: &mut Vec<u8>) {
    use OwnedAnnularJoinCause as O;
    match value {
        O::AuthoredIdentity {
            quantity,
            actual,
            expected,
        } => {
            bytes.push(0);
            encode_joined_text(quantity, bytes);
            bytes.extend([actual.0, expected.0].map(u64::to_le_bytes).concat());
        }
        O::Version { actual, expected } => {
            bytes.push(1);
            bytes.extend([*actual, *expected].map(u32::to_le_bytes).concat());
        }
        O::NameBudget { maximum, measured } => {
            bytes.extend([2, u8::from(measured.is_some())]);
            let words = [*maximum as u64, measured.unwrap_or(0) as u64];
            bytes.extend(words.map(u64::to_le_bytes).concat());
        }
        O::Callback(failure) => {
            bytes.push(3);
            crate::sketch_region_prism::diagnostic_envelope::append_native_failure(failure, bytes);
        }
        O::Cancelled { freeze_phase } => {
            bytes.extend([4, u8::from(freeze_phase.is_some())]);
            bytes.extend(freeze_phase.map(freeze_tag));
        }
    }
}
