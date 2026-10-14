// SPDX-License-Identifier: GPL-2.0-only
//! Retain owned numerical witnesses/work intact; report and captured-store projection remain explicit gaps.
use super::*;
use crate::sketch_region_candidates::CandidateCause;
pub(super) fn project(
    cause: CandidateCause,
) -> ProjectionCheck<OwnedPrismCandidateCause, CandidateCause> {
    match cause {
        CandidateCause::Mapped(value) => Ok(OwnedPrismCandidateCause::Native(value)),
        CandidateCause::Cancelled(value) => Ok(OwnedPrismCandidateCause::Cancelled(value)),
        CandidateCause::Numerical(value) => Ok(OwnedPrismCandidateCause::Numerical(value)),
        CandidateCause::CapturedNumerical(value) => {
            Ok(OwnedPrismCandidateCause::CapturedNumerical(value))
        }
        CandidateCause::MappedReport(_) => {
            pending(cause, PendingPrismRefusalPayload::CandidateReport)
        }
        CandidateCause::Incomplete(_) => pending(cause, PendingPrismRefusalPayload::CandidateGap),
        CandidateCause::CapturedOperation(_) => pending(
            cause,
            PendingPrismRefusalPayload::CandidateCapturedOperation,
        ),
    }
}
fn pending(
    cause: CandidateCause,
    payload: PendingPrismRefusalPayload,
) -> ProjectionCheck<OwnedPrismCandidateCause, CandidateCause> {
    let retained = RetainedProjection::pending(cause, PrismRefusalPath::Candidate, payload);
    Err(retained)
}
