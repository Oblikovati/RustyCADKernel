// SPDX-License-Identifier: GPL-2.0-only
//! Move an owned history finding out of its actual publication carrier when it is complete.
use super::history_codec::finding_bytes;
use super::*;

/// Outer header of a consumed publication refusal.
pub(super) struct HistoryHeader {
    pub operation: AuthoredPrismId,
    pub phase: PrismPublicationPhase,
    pub class: PrismPublicationClass,
}
type Taken<'a> = Result<(HistoryHeader, OwnedPublicationHistory), Box<PrismPublicationFailure<'a>>>;
/// Move an attributed, codec-supported finding out of its actual carrier; any other refusal is
/// returned unchanged for the general projection. A local owner-bearing diagnostic never moves.
/// Example: `take(packet.into_failure(), gaps.is_empty())`.
pub(super) fn take(original: Box<PrismPublicationFailure<'_>>, attributed: bool) -> Taken<'_> {
    match encoded(&original, attributed) {
        Some(encoded) => Ok(split(*original, encoded)),
        None => Err(original),
    }
}
fn encoded(original: &PrismPublicationFailure<'_>, attributed: bool) -> Option<Vec<u8>> {
    let PrismPublicationCause::History(failure) = &original.cause else {
        return None;
    };
    let local = failure.local_diagnostic.is_some();
    (attributed && !local)
        .then(|| finding_bytes(&failure.finding).ok())
        .flatten()
}
fn split(
    original: PrismPublicationFailure<'_>,
    encoded: Vec<u8>,
) -> (HistoryHeader, OwnedPublicationHistory) {
    let (operation, phase, class) = (original.operation, original.phase, original.class);
    let PrismPublicationCause::History(failure) = original.cause else {
        unreachable!("checked history carrier");
    };
    let header = HistoryHeader {
        operation,
        phase,
        class,
    };
    let finding = failure.finding;
    (header, OwnedPublicationHistory { finding, encoded })
}
