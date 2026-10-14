// SPDX-License-Identifier: GPL-2.0-only
//! One owning projection, followed by callback-free live operand rebinding; no body validity promotion.
use super::{
    admission, candidates, census_settings, current_history, failure::*, nesting, outcome::*,
    pair_capture, replay, request::*, sources, work::*,
};

/// Inspect one already constructed nested two-loop partition; `inspect_two_loop_region(&request,&mut progress)`.
/// The inspected region domain supplies no complete prism/history/tolerance publication certificate.
pub(crate) fn inspect_two_loop_region<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    progress: &mut dyn TwoLoopRegionProgress,
) -> TwoLoopRegionOutcome<'a> {
    match inspect(request, progress) {
        Ok(proof) => TwoLoopRegionOutcome::Inspected(Box::new(proof)),
        Err(failure) => outcome(failure),
    }
}
fn inspect<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    progress: &mut dyn TwoLoopRegionProgress,
) -> TwoLoopCheck<'a, TwoLoopRegionCertificate> {
    let mut work = TwoLoopWork::new(request.identities.operation, request.requested, progress);
    let (sources, domains, census) = initial(request, &mut work)?;
    work.poll(TwoLoopRegionStage::Nesting)?;
    let nesting = nesting::capture(request, &sources, &mut work)?;
    work.poll(TwoLoopRegionStage::Membership)?;
    let candidates = candidates::capture(request, &sources, &nesting, &mut work)?;
    current_history::inspect(request, &mut work)?;
    let binding = replay::binding(request, &sources, &domains, &candidates, &census);
    let replay = replay::complete(&binding, &nesting, work.retained_units);
    let proof = TwoLoopRegionCertificate {
        operation: request.identities.operation,
        selected: request.identities.selected,
        requested: request.requested,
        sources,
        domains,
        candidates,
        nesting,
        census,
        retained_units: work.retained_units,
        binding,
        replay,
    };
    work.poll(TwoLoopRegionStage::Commit)?;
    rebind_two_loop_region(request, &proof)?;
    Ok(proof)
}
fn initial<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<
    'a,
    (
        Vec<TwoLoopSourceProof>,
        Vec<super::pair_types::TwoLoopPairProof>,
        census_settings::TwoLoopCensusSettings,
    ),
> {
    work.poll(TwoLoopRegionStage::Admission)?;
    admission::inspect(request, work)?;
    super::availability::inspect(request, work)?;
    super::history_budget::admit(request, work)?;
    work.poll(TwoLoopRegionStage::Sources)?;
    let sources = sources::capture(request, work)?;
    work.poll(TwoLoopRegionStage::Census)?;
    let rank = super::plane::inspect(request, &sources, work)?;
    // Rank evaluation polls the caller; re-admit live evaluators after its last callback.
    sources::preflight(request, work)?;
    let census = census_settings::capture(&request.bridge.census, rank, work)?;
    let names = sources::census_names(request, work)?;
    let domains = pair_capture::domains(&request.bridge.census, names, work)?;
    Ok((sources, domains, census))
}
/// Check current live operands without a user callback; `rebind_two_loop_region(&request,&proof)?`.
/// Exact input equality permits reuse of the already owned winding proof; revision labels are insufficient.
pub(crate) fn rebind_two_loop_region<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    proof: &TwoLoopRegionCertificate,
) -> TwoLoopCheck<'a, ()> {
    let mut progress = SilentRebind;
    let mut work = TwoLoopWork::new(
        request.identities.operation,
        request.requested,
        &mut progress,
    );
    let (sources, domains, census) = initial(request, &mut work)?;
    work.units(
        "retained nesting proof operands",
        proof.nesting.captured_units,
    )?;
    let candidates = candidates::capture(request, &sources, &proof.nesting, &mut work)?;
    current_history::inspect(request, &mut work)?;
    let binding = replay::binding(request, &sources, &domains, &candidates, &census);
    if binding != proof.binding || work.retained_units != proof.retained_units {
        return Err(work.failure(TwoLoopRegionCause::Stale {
            operand: "complete current source/candidate/domain binding",
        }));
    }
    Ok(())
}
struct SilentRebind;
impl TwoLoopRegionProgress for SilentRebind {
    fn checkpoint(
        &mut self,
        _: TwoLoopRegionStage,
    ) -> Result<bool, rustycad_tkernel::standard::Failure> {
        Ok(false)
    }
}
fn outcome(failure: Box<TwoLoopRegionFailure<'_>>) -> TwoLoopRegionOutcome<'_> {
    match super::failure_class::classify(&failure) {
        super::failure_class::RefusalClass::Cancelled => TwoLoopRegionOutcome::Cancelled(failure),
        super::failure_class::RefusalClass::Failed => TwoLoopRegionOutcome::Failed(failure),
        _ => TwoLoopRegionOutcome::Incomplete(failure),
    }
}
