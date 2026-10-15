// SPDX-License-Identifier: GPL-2.0-only
//! The final callback precedes the shared annular join rebind and history-only operand recapture.
use super::result::*;
use super::*;

/// Inspect bounded actual annular history; `inspect_annular_prism_history(&request, |_| false)`.
/// Success certifies history only; it never publishes or validates a body.
pub(in crate::sketch_region_prism) fn inspect_annular_prism_history<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    mut cancelled: impl FnMut(PrismHistoryPhase) -> bool,
) -> AnnularHistoryOutcome<'a> {
    if cancelled(PrismHistoryPhase::Admission) {
        return cancelled_outcome(request, PrismHistoryPhase::Admission);
    }
    if let Err(error) =
        admit(request).and_then(|_| rebind_join(request, PrismHistoryPhase::Capture))
    {
        return declined(error);
    }
    if cancelled(PrismHistoryPhase::Capture) {
        return cancelled_outcome(request, PrismHistoryPhase::Capture);
    }
    match project(request, PrismHistoryPhase::Capture) {
        Ok(captured) => commit(request, captured, cancelled),
        Err(error) => declined(error),
    }
}
fn commit<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    captured: AnnularCompleteHistory,
    mut cancelled: impl FnMut(PrismHistoryPhase) -> bool,
) -> AnnularHistoryOutcome<'a> {
    if cancelled(PrismHistoryPhase::Commit) {
        return cancelled_outcome(request, PrismHistoryPhase::Commit);
    }
    match rebind_annular_prism_history(request, &captured) {
        Ok(()) => AnnularHistoryOutcome::Inspected(Box::new(captured)),
        Err(error) => declined(error),
    }
}
/// Project history over a caller's fresh annular join without callbacks or another join rebind.
/// The caller must later run its shared join rebind and `reproject_annular_prism_history`.
/// Example: `inspect_bound_annular_prism_history(&request)` inside a publication assembler.
pub(in crate::sketch_region_prism) fn inspect_bound_annular_prism_history<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
) -> AnnularHistoryOutcome<'a> {
    match admit(request).and_then(|_| project(request, PrismHistoryPhase::Capture)) {
        Ok(value) => AnnularHistoryOutcome::Inspected(Box::new(value)),
        Err(error) => declined(error),
    }
}
/// Rebind standalone history after the last external callback; `rebind_annular_prism_history(&r, &h)`.
pub(in crate::sketch_region_prism) fn rebind_annular_prism_history<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    history: &AnnularCompleteHistory,
) -> Result<(), AnnularHistoryFailure<'a>> {
    admit(request)?;
    rebind_join(request, PrismHistoryPhase::Commit)?;
    reproject_annular_prism_history(request, history)
}
/// History-only callback-free recapture after the caller's shared join rebind; the complete
/// replay bytes, including rejected/intermediate/query observations absent from join, must match.
/// Example: `reproject_annular_prism_history(&request, &history)`.
pub(in crate::sketch_region_prism) fn reproject_annular_prism_history<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    history: &AnnularCompleteHistory,
) -> Result<(), AnnularHistoryFailure<'a>> {
    admit(request)?;
    if !history.binds(request.joined) {
        return Err(failure(
            request,
            PrismHistoryPhase::Commit,
            PrismHistoryCause::Changed {
                operand: "same complete annular join replay",
            },
        ));
    }
    let current = project(request, PrismHistoryPhase::Commit)?;
    super::super::complete_history::observed_values::equal(
        &current.replay_bytes(),
        &history.replay_bytes(),
        "all actual annular history operands after final callback",
    )
    .map_err(|cause| failure(request, PrismHistoryPhase::Commit, cause))
}
fn admit<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
) -> Result<usize, AnnularHistoryFailure<'a>> {
    let measured = super::admission::admit(request)
        .map_err(|cause| failure(request, PrismHistoryPhase::Admission, cause))?;
    super::super::complete_history::diagnostics::admit_native_observations(
        &request.native_inputs(),
    )
    .map_err(AnnularHistoryFailure::from)?;
    Ok(measured)
}
fn project<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    phase: PrismHistoryPhase,
) -> Result<AnnularCompleteHistory, AnnularHistoryFailure<'a>> {
    let measured =
        super::admission::admit(request).map_err(|cause| failure(request, phase, cause))?;
    super::capture::capture(request)
        .map(|captured| super::replay::finish(request, captured, measured))
        .map_err(|cause| failure(request, phase, cause))
}
fn rebind_join<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    phase: PrismHistoryPhase,
) -> Result<(), AnnularHistoryFailure<'a>> {
    super::super::annular_join::rebind_annular_frozen_domain(
        request.region,
        request.receipt,
        request.publication,
        request.joined,
    )
    .map_err(|error| {
        let cause = super::super::complete_history::diagnostics::annular_join_cause(&error);
        AnnularHistoryFailure {
            join: Some(error),
            ..failure(request, phase, cause)
        }
    })
}
fn failure<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    phase: PrismHistoryPhase,
    cause: PrismHistoryCause,
) -> AnnularHistoryFailure<'a> {
    super::super::complete_history::operation::native_failure(
        request.publication.operation,
        phase,
        cause,
    )
    .into()
}
fn declined(error: AnnularHistoryFailure<'_>) -> AnnularHistoryOutcome<'_> {
    match error.history.finding.class {
        PrismHistoryFailureClass::Incomplete => AnnularHistoryOutcome::Incomplete(error),
        PrismHistoryFailureClass::Failed => AnnularHistoryOutcome::Failed(error),
    }
}
fn cancelled_outcome<'a>(
    request: &AnnularHistoryRequest<'a, '_, '_, '_>,
    phase: PrismHistoryPhase,
) -> AnnularHistoryOutcome<'a> {
    AnnularHistoryOutcome::Cancelled {
        operation: request.publication.operation,
        phase,
    }
}
