// SPDX-License-Identifier: GPL-2.0-only
//! Conditional history shares the join and leaves the sole final live check to publication.
use super::{progress::*, request::*, result::*, stage_error::*};
use crate::sketch_region_prism::frozen::{complete_history::*, proof_join::JoinedPrismProof};

pub(super) fn capture<'a>(
    request: &PrismPublicationRequest<'a>,
    join: &JoinedPrismProof,
    callbacks: &mut PublicationCallbacks<'_>,
) -> PublicationCheck<'a, Box<PrismCompleteHistory>> {
    request.callback(PrismPublicationPhase::History, callbacks)?;
    let proof = inspect_bound_prism_history(&request.history_request(join));
    let proof = accepted_history(request, proof)?;
    if !proof.binds(join) {
        return Err(request.stage_failure(binding_failure(
            PrismPublicationPhase::History,
            "complete history",
        )));
    }
    Ok(proof)
}
fn accepted_history<'a>(
    request: &PrismPublicationRequest<'a>,
    outcome: PrismHistoryOutcome<'a>,
) -> PublicationCheck<'a, Box<PrismCompleteHistory>> {
    match outcome {
        PrismHistoryOutcome::Inspected(proof) => Ok(proof),
        PrismHistoryOutcome::Failed(cause) | PrismHistoryOutcome::Incomplete(cause) => {
            Err(request.history_failure(cause, PrismPublicationPhase::History))
        }
        PrismHistoryOutcome::Cancelled { operation, phase } => Err(request.failure(
            PrismPublicationPhase::History,
            PrismPublicationClass::Cancelled,
            PrismPublicationCause::HistoryCancelled { operation, phase },
        )),
    }
}
pub(super) fn reproject<'a>(
    request: &PrismPublicationRequest<'a>,
    result: &PublishedNamedPrism,
) -> PublicationCheck<'a, ()> {
    reproject_complete_prism_history(&request.history_request(&result.join), &result.history)
        .map_err(|cause| request.history_failure(cause, PrismPublicationPhase::Commit))
}
