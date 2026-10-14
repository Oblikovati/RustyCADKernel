// SPDX-License-Identifier: GPL-2.0-only
//! One operation-local entry owns every reachable result value; callbacks cannot publish a prefix.
use super::super::PrismReceipt;
use super::{
    bindings::*, coverage::capture_coverage, geometry::*, graph::capture_graph,
    history::capture_history, types::*,
};

/// Capture diagnostic geometry only; example: `freeze_prism(&receipt, &authored, &mut progress)`.
/// Call with exclusive operation-local geometry; concurrent alias mutation is unsupported.
/// Unsupported identity/evidence remains incomplete, never a valid-body result.
pub(crate) fn freeze_prism(
    receipt: &PrismReceipt,
    bindings: &PrismPublicationBindings,
    progress: &mut impl FreezeProgress,
) -> PrismFreezeOutcome {
    let mut phase = FreezePhase::Admission;
    match execute(receipt, bindings, progress, &mut phase) {
        Ok(projection) => PrismFreezeOutcome::Captured(projection),
        Err(FreezeStop::Cancelled) => PrismFreezeOutcome::Cancelled {
            operation: bindings.operation,
            phase,
        },
        Err(FreezeStop::Cause(cause)) => classify_failure(bindings.operation, phase, cause),
    }
}
enum FreezeStop {
    Cancelled,
    Cause(PrismFreezeCause),
}
impl From<PrismFreezeCause> for FreezeStop {
    fn from(cause: PrismFreezeCause) -> Self {
        Self::Cause(cause)
    }
}
fn classify_failure(
    operation: AuthoredPrismId,
    phase: FreezePhase,
    cause: PrismFreezeCause,
) -> PrismFreezeOutcome {
    let native = matches!(cause, PrismFreezeCause::Native(_));
    let failure = PrismFreezeFailure {
        operation,
        phase,
        entity: None,
        cause,
    };
    if native {
        PrismFreezeOutcome::Failed(failure)
    } else {
        PrismFreezeOutcome::Incomplete(failure)
    }
}
fn execute(
    receipt: &PrismReceipt,
    bindings: &PrismPublicationBindings,
    progress: &mut impl FreezeProgress,
    phase: &mut FreezePhase,
) -> Result<FrozenPrismProjection, FreezeStop> {
    checkpoint(progress, phase, FreezePhase::Admission)?;
    super::admission::admit_receipt(receipt)?;
    checkpoint(progress, phase, FreezePhase::Identity)?;
    let bound = bind_projection(receipt, bindings)?;
    super::domain::check_count(bound.contexts.len())?;
    let projection = capture_phases(&bound, progress, phase)?;
    checkpoint(progress, phase, FreezePhase::Commit)?;
    check_source_snapshot(&bound)?;
    check_final_capture(&bound, &projection)?;
    Ok(projection)
}
fn capture_phases(
    bound: &BoundProjection<'_>,
    progress: &mut impl FreezeProgress,
    phase: &mut FreezePhase,
) -> Result<FrozenPrismProjection, FreezeStop> {
    checkpoint(progress, phase, FreezePhase::Geometry)?;
    check_source_snapshot(bound)?;
    let faces = capture_faces(bound)?;
    checkpoint(progress, phase, FreezePhase::Incidence)?;
    check_current_contexts(bound)?;
    let incidence = capture_graph(bound)?;
    let nodes = capture_nodes(bound)?;
    checkpoint(progress, phase, FreezePhase::Evidence)?;
    Ok(finish_projection(bound, nodes, faces, incidence)?)
}
fn checkpoint(
    progress: &mut impl FreezeProgress,
    phase: &mut FreezePhase,
    next: FreezePhase,
) -> Result<(), FreezeStop> {
    *phase = next;
    match progress.checkpoint(next) {
        Ok(false) => Ok(()),
        Ok(true) => Err(FreezeStop::Cancelled),
        Err(failure) => Err(PrismFreezeCause::Native(failure).into()),
    }
}
fn finish_projection(
    bound: &BoundProjection<'_>,
    nodes: Vec<FrozenPrismNode>,
    faces: Vec<FrozenPrismFace>,
    graph: super::graph::FrozenGraphIncidence,
) -> CaptureResult<FrozenPrismProjection> {
    Ok(FrozenPrismProjection {
        version: 1,
        capture_policy: super::domain::fixed_policy(),
        operation: bound.bindings.operation,
        region: bound.bindings.selected_region,
        endpoints: super::endpoints::certify_endpoints(&nodes, &graph)?,
        nodes,
        faces,
        graph,
        uses: capture_uses(bound)?,
        history: capture_history(bound)?,
        coverage: capture_coverage(bound)?,
        limits: capture_limits(bound.receipt),
    })
}
fn capture_uses(bound: &BoundProjection<'_>) -> CaptureResult<Vec<FrozenPrismUse>> {
    let mut uses = (0..bound.contexts.len())
        .map(|i| capture_use(bound, i))
        .collect::<Result<Vec<_>, _>>()?;
    uses.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(uses)
}
fn capture_use(bound: &BoundProjection<'_>, index: usize) -> CaptureResult<FrozenPrismUse> {
    let context = &bound.contexts[index];
    Ok(FrozenPrismUse {
        key: bound.use_names[index].clone(),
        orientation: context.occurrence.orientation(),
        placement: super::history::freeze_placement(context.occurrence.location())?,
        children: child_keys(bound, &context.child_slots),
    })
}
fn child_keys(bound: &BoundProjection<'_>, path: &[usize]) -> Vec<FrozenUseKey> {
    bound
        .contexts
        .iter()
        .enumerate()
        .filter(|(_, child)| {
            child.child_slots.len() == path.len() + 1 && child.child_slots.starts_with(path)
        })
        .map(|(index, _)| bound.use_names[index].clone())
        .collect()
}
fn capture_limits(receipt: &PrismReceipt) -> FrozenPrismLimits {
    let requested = receipt.limits.requested;
    let effective = &receipt.limits.effective;
    let vector = requested.extrusion;
    FrozenPrismLimits {
        seed: requested.seed,
        extrusion: [vector.x(), vector.y(), vector.z()],
        roots: requested.roots,
        spatial_limit: requested.spatial_limit,
        parameter_limit: requested.parameter_limit,
        effective_seed: effective.seed,
        effective_extrusion: effective.extrusion.map(|v| [v.x(), v.y(), v.z()]),
        effective_roots: effective.roots,
        effective_spatial: effective.spatial_limit,
        effective_parameter: effective.parameter_limit,
        achieved_error: receipt.limits.achieved_error,
    }
}
fn check_final_capture(
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
) -> CaptureResult<()> {
    let fresh = bind_projection(bound.receipt, bound.bindings)?;
    check_current_contexts(bound)?;
    check_source_snapshot(&fresh)?;
    let incidence = capture_graph(&fresh)?;
    let nodes = capture_nodes(&fresh)?;
    let recaptured = finish_projection(&fresh, nodes, capture_faces(&fresh)?, incidence)?;
    if recaptured.replay_bytes() != projection.replay_bytes() {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "full geometry/incidence/history/evidence changed during capture",
        });
    }
    Ok(())
}
fn check_current_contexts(bound: &BoundProjection<'_>) -> CaptureResult<()> {
    let fresh = bind_projection(bound.receipt, bound.bindings)?;
    if fresh.contexts.len() != bound.contexts.len()
        || !fresh
            .contexts
            .iter()
            .zip(&bound.contexts)
            .all(|(a, b)| super::identities::same_context(a, b))
        || fresh.names != bound.names
        || fresh.use_names != bound.use_names
    {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "actual root containment or authored context changed during capture",
        });
    }
    Ok(())
}
