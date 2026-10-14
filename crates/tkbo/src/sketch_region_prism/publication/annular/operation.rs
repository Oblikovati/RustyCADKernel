// SPDX-License-Identifier: GPL-2.0-only
//! The terminal callback precedes the callback-free current source rebind and history reprojection;
//! a prospective result stays local until both pass.
use super::stages::{AnnularStageStop, StageCheck};
use super::*;

/// Assemble one named annular prism; `publish_annular_prism(&request, evidence, &mut progress)`.
/// Join, ordered validation, complete history and measures bind one source; no partial body.
pub(in crate::sketch_region_prism) fn publish_annular_prism<'a>(
    request: &AnnularPublicationRequest<'a, '_, '_>,
    embedding: AnnularEmbeddingEvidence,
    progress: &mut dyn PrismPublicationProgress,
) -> AnnularPublicationCheck<'a, PublishedAnnularPrism> {
    admit(request)?;
    let mut callbacks = PublicationCallbacks::new(progress);
    let mut result =
        construct(request, embedding, &mut callbacks).map_err(|s| failure(request, s))?;
    result.replay = super::replay::encode(&result);
    callback(PrismPublicationPhase::Commit, &mut callbacks).map_err(|s| failure(request, s))?;
    rebind(request, &result).map_err(|s| failure(request, s))?;
    Ok(result)
}
fn admit<'a>(request: &AnnularPublicationRequest<'a, '_, '_>) -> AnnularPublicationCheck<'a, ()> {
    let actual = request.requested.version;
    if actual == 1 {
        return Ok(());
    }
    let cause = AnnularPublicationCause::UnsupportedVersion {
        actual,
        expected: 1,
    };
    let (phase, class) = (
        PrismPublicationPhase::Admission,
        PrismPublicationClass::Incomplete,
    );
    Err(failure(
        request,
        AnnularStageStop {
            phase,
            class,
            cause,
        },
    ))
}
fn construct<'a>(
    request: &AnnularPublicationRequest<'a, '_, '_>,
    embedding: AnnularEmbeddingEvidence,
    callbacks: &mut PublicationCallbacks<'_>,
) -> StageCheck<'a, PublishedAnnularPrism> {
    let join = stages::join(request, embedding, callbacks)?;
    let validation = stages::validation(&join, request.requested.validation, callbacks)?;
    let history = history(request, &join, callbacks)?;
    let measures = stages::measures(&join, request.requested.measures, callbacks)?;
    Ok(PublishedAnnularPrism {
        join,
        validation,
        history,
        measures,
        requested: request.requested,
        replay: Vec::new(),
    })
}
fn accepted(outcome: AnnularHistoryOutcome<'_>) -> StageCheck<'_, Box<AnnularCompleteHistory>> {
    let phase = PrismPublicationPhase::History;
    match outcome {
        AnnularHistoryOutcome::Inspected(proof) => Ok(proof),
        AnnularHistoryOutcome::Failed(cause) | AnnularHistoryOutcome::Incomplete(cause) => {
            Err(history_stop(cause, phase))
        }
        AnnularHistoryOutcome::Cancelled { operation, phase } => Err(cancelled(operation, phase)),
    }
}
fn cancelled<'a>(operation: AuthoredPrismId, phase: PrismHistoryPhase) -> AnnularStageStop<'a> {
    let cause = AnnularPublicationCause::HistoryCancelled { operation, phase };
    let class = PrismPublicationClass::Cancelled;
    let phase = PrismPublicationPhase::History;
    AnnularStageStop {
        phase,
        class,
        cause,
    }
}
/// Conditional history over the shared join; the sole live rebind is left to commit.
fn history<'a>(
    request: &AnnularPublicationRequest<'a, '_, '_>,
    join: &AnnularFrozenDomain,
    callbacks: &mut PublicationCallbacks<'_>,
) -> StageCheck<'a, Box<AnnularCompleteHistory>> {
    let phase = PrismPublicationPhase::History;
    callback(phase, callbacks)?;
    let proof = accepted(inspect_bound_annular_prism_history(&request.history(join)))?;
    stages::bound(proof.binds(join), phase, "complete annular history")?;
    Ok(proof)
}
fn history_stop(
    failure: AnnularHistoryFailure<'_>,
    phase: PrismPublicationPhase,
) -> AnnularStageStop<'_> {
    let class = match failure.finding().class {
        PrismHistoryFailureClass::Failed => PrismPublicationClass::Failed,
        PrismHistoryFailureClass::Incomplete => PrismPublicationClass::Incomplete,
    };
    let cause = AnnularPublicationCause::History(Box::new(failure));
    AnnularStageStop {
        phase,
        class,
        cause,
    }
}
fn callback<'a>(
    phase: PrismPublicationPhase,
    callbacks: &mut PublicationCallbacks<'_>,
) -> StageCheck<'a, ()> {
    callbacks.cancelled(phase);
    stages::finish(Ok(()), callbacks)
}
/// Callback-free: the shared live join rebind, then history-only full-byte reprojection.
fn rebind<'a>(
    request: &AnnularPublicationRequest<'a, '_, '_>,
    result: &PublishedAnnularPrism,
) -> StageCheck<'a, ()> {
    let phase = PrismPublicationPhase::Commit;
    let (region, receipt, names) = (request.region, request.receipt, request.bindings);
    rebind_annular_frozen_domain(region, receipt, names, &result.join)
        .map_err(|failure| stages::join_stop(failure, phase))?;
    reproject_annular_prism_history(&request.history(&result.join), &result.history)
        .map_err(|failure| history_stop(failure, phase))
}
fn failure<'a>(
    request: &AnnularPublicationRequest<'a, '_, '_>,
    stop: AnnularStageStop<'a>,
) -> Box<AnnularPublicationFailure<'a>> {
    Box::new(AnnularPublicationFailure {
        operation: request.bindings.operation,
        requested: request.requested,
        phase: stop.phase,
        class: stop.class,
        cause: stop.cause,
    })
}
