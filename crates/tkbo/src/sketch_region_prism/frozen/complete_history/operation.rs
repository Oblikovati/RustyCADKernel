// SPDX-License-Identifier: GPL-2.0-only
//! The final callback precedes both shared live rebind and complete history operand recapture.
use super::*;
/// Inspect bounded actual history; `inspect_complete_prism_history(&request, |_|false)` never publishes a body.
pub(crate) fn inspect_complete_prism_history<'a>(
    request: &PrismHistoryRequest<'a, '_>,
    mut cancelled: impl FnMut(PrismHistoryPhase) -> bool,
) -> PrismHistoryOutcome<'a> {
    if cancelled(PrismHistoryPhase::Admission) {
        return cancelled_outcome(request, PrismHistoryPhase::Admission);
    }
    if let Err(error) = admit(request) {
        return declined(error);
    }
    if let Err(error) = rebind_join(request, PrismHistoryPhase::Capture) {
        return declined(error);
    }
    if cancelled(PrismHistoryPhase::Capture) {
        return cancelled_outcome(request, PrismHistoryPhase::Capture);
    }
    let captured = match project(request, PrismHistoryPhase::Capture) {
        Ok(value) => value,
        Err(error) => return declined(error),
    };
    if cancelled(PrismHistoryPhase::Commit) {
        return cancelled_outcome(request, PrismHistoryPhase::Commit);
    }
    if let Err(error) = rebind_complete_prism_history(request, &captured) {
        return declined(error);
    }
    PrismHistoryOutcome::Inspected(Box::new(captured))
}
/// Rebind standalone history after the last external callback; `rebind_complete_prism_history(&request,&history)`.
pub(crate) fn rebind_complete_prism_history<'a>(
    request: &PrismHistoryRequest<'a, '_>,
    history: &PrismCompleteHistory,
) -> Result<(), PrismHistoryFailure<'a>> {
    admit(request)?;
    rebind_join(request, PrismHistoryPhase::Commit)?;
    reproject_complete_prism_history(request, history)
}
/// History-only final projection after a caller's shared join rebind; `reproject_complete_prism_history(&request,&history)`.
/// The caller must perform the shared live join check immediately before this function with no callback between them.
pub(crate) fn reproject_complete_prism_history<'a>(
    request: &PrismHistoryRequest<'a, '_>,
    history: &PrismCompleteHistory,
) -> Result<(), PrismHistoryFailure<'a>> {
    admit(request)?;
    if !history.binds(request.joined) {
        return Err(failure(
            request,
            PrismHistoryPhase::Commit,
            PrismHistoryCause::Changed {
                operand: "same complete joined proof replay",
            },
        ));
    }
    let current = project(request, PrismHistoryPhase::Commit)?;
    observed_values::equal(
        &current.replay_bytes(),
        &history.replay_bytes(),
        "all actual history operands after final callback",
    )
    .map_err(|cause| failure(request, PrismHistoryPhase::Commit, cause))
}
fn project<'a>(
    request: &PrismHistoryRequest<'a, '_>,
    phase: PrismHistoryPhase,
) -> Result<PrismCompleteHistory, PrismHistoryFailure<'a>> {
    capture::capture(request)
        .and_then(|captured| replay::finish(request, captured))
        .map_err(|cause| failure(request, phase, cause))
}
fn admit<'a>(request: &PrismHistoryRequest<'a, '_>) -> Result<(), PrismHistoryFailure<'a>> {
    admission::admit(request)
        .map_err(|cause| failure(request, PrismHistoryPhase::Admission, cause))?;
    diagnostics::admit_observations(request)
}
fn rebind_join<'a>(
    request: &PrismHistoryRequest<'a, '_>,
    phase: PrismHistoryPhase,
) -> Result<(), PrismHistoryFailure<'a>> {
    super::super::proof_join::rebind_joined_prism_proof(
        request.bridge,
        request.sources,
        request.receipt,
        request.publication,
        request.joined,
    )
    .map_err(|error| diagnostics::join_failure(request, phase, error))
}
pub(super) fn failure<'a>(
    request: &PrismHistoryRequest<'a, '_>,
    phase: PrismHistoryPhase,
    cause: PrismHistoryCause,
) -> PrismHistoryFailure<'a> {
    let class = cause.failure_class();
    let entity = cause.primary_use();
    PrismHistoryFailure {
        finding: PrismHistoryFinding {
            operation: request.publication.operation,
            phase,
            entity,
            class,
            cause,
        },
        local_diagnostic: None,
    }
}
fn declined(error: PrismHistoryFailure<'_>) -> PrismHistoryOutcome<'_> {
    match error.finding.class {
        PrismHistoryFailureClass::Incomplete => PrismHistoryOutcome::Incomplete(error),
        PrismHistoryFailureClass::Failed => PrismHistoryOutcome::Failed(error),
    }
}
fn cancelled_outcome<'a>(
    request: &PrismHistoryRequest<'a, '_>,
    phase: PrismHistoryPhase,
) -> PrismHistoryOutcome<'a> {
    PrismHistoryOutcome::Cancelled {
        operation: request.publication.operation,
        phase,
    }
}

/// Project history over a caller's fresh join without callbacks or another shared rebind.
/// Example: `inspect_bound_prism_history(&request)` inside an assembler that subsequently rebinds join and history.
/// This conditional entry requires the caller's final shared live join check and history-only reprojection before publication.
pub(crate) fn inspect_bound_prism_history<'a>(
    request: &PrismHistoryRequest<'a, '_>,
) -> PrismHistoryOutcome<'a> {
    if let Err(error) = admit(request) {
        return declined(error);
    }
    match project(request, PrismHistoryPhase::Capture) {
        Ok(value) => PrismHistoryOutcome::Inspected(Box::new(value)),
        Err(error) => declined(error),
    }
}
