// SPDX-License-Identifier: GPL-2.0-only
//! One operation-local entry owns every reachable result value; callbacks cannot publish a prefix.
use super::super::{PrismCompletion, PrismReceipt};
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
    admit_receipt(receipt)?;
    checkpoint(progress, phase, FreezePhase::Identity)?;
    let bound = bind_projection(receipt, bindings)?;
    if bound.contexts.len() > 4096 {
        return Err(missing("at most 4096 directed occurrences").into());
    }
    checkpoint(progress, phase, FreezePhase::Geometry)?;
    check_source_snapshot(&bound)?;
    let faces = capture_faces(&bound)?;
    checkpoint(progress, phase, FreezePhase::Incidence)?;
    check_current_contexts(&bound)?;
    let (graph, incidence) = capture_graph(&bound)?;
    let nodes = capture_nodes(&bound, &graph)?;
    super::endpoints::certify_endpoints(&nodes, &incidence)?;
    checkpoint(progress, phase, FreezePhase::Evidence)?;
    let projection = finish_projection(&bound, nodes, faces, incidence)?;
    checkpoint(progress, phase, FreezePhase::Commit)?;
    check_source_snapshot(&bound)?;
    check_final_capture(&bound, &projection)?;
    Ok(projection)
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
fn admit_receipt(receipt: &PrismReceipt) -> Result<(), PrismFreezeCause> {
    if !matches!(receipt.completion, PrismCompletion::Incomplete) {
        return Err(missing("completed private diagnostic transaction"));
    }
    let diagnostic = receipt
        .diagnostic
        .as_ref()
        .ok_or(missing("diagnostic body"))?;
    if !diagnostic.transport_gaps.is_empty() {
        return Err(missing("complete observed history transport"));
    }
    if receipt.cancelled_query.is_some() {
        return Err(missing("uncancelled source selection"));
    }
    Ok(())
}
fn finish_projection(
    bound: &BoundProjection<'_>,
    nodes: Vec<FrozenPrismNode>,
    faces: Vec<FrozenPrismFace>,
    graph: super::graph::FrozenGraphIncidence,
) -> Result<FrozenPrismProjection, PrismFreezeCause> {
    Ok(FrozenPrismProjection {
        version: 1,
        operation: bound.bindings.operation,
        region: bound.bindings.selected_region,
        nodes,
        faces,
        graph,
        uses: capture_uses(bound),
        history: capture_history(bound)?,
        coverage: capture_coverage(bound)?,
        limits: capture_limits(bound.receipt),
    })
}
fn capture_uses(bound: &BoundProjection<'_>) -> Vec<FrozenPrismUse> {
    let mut uses = Vec::new();
    for (index, context) in bound.contexts.iter().enumerate() {
        let children = bound
            .contexts
            .iter()
            .enumerate()
            .filter(|(_, child)| {
                child.child_slots.len() == context.child_slots.len() + 1
                    && child.child_slots.starts_with(&context.child_slots)
            })
            .map(|(i, _)| bound.use_names[i].clone())
            .collect();
        uses.push(FrozenPrismUse {
            key: bound.use_names[index].clone(),
            orientation: context.occurrence.orientation(),
            children,
        });
    }
    uses.sort_by(|a, b| a.key.cmp(&b.key));
    uses
}
fn capture_limits(receipt: &PrismReceipt) -> FrozenPrismLimits {
    let requested = receipt.limits.requested;
    let effective = &receipt.limits.effective;
    FrozenPrismLimits {
        seed: requested.seed,
        extrusion: [
            requested.extrusion.x(),
            requested.extrusion.y(),
            requested.extrusion.z(),
        ],
        roots: requested.roots,
        spatial_limit: requested.spatial_limit,
        parameter_limit: requested.parameter_limit,
        effective_spatial: effective.spatial_limit,
        effective_parameter: effective.parameter_limit,
        achieved_error: receipt.limits.achieved_error,
    }
}
fn check_final_capture(
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
) -> Result<(), PrismFreezeCause> {
    let fresh = bind_projection(bound.receipt, bound.bindings)?;
    check_current_contexts(bound)?;
    check_source_snapshot(&fresh)?;
    let (graph, incidence) = capture_graph(&fresh)?;
    let nodes = capture_nodes(&fresh, &graph)?;
    super::endpoints::certify_endpoints(&nodes, &incidence)?;
    let recaptured = finish_projection(&fresh, nodes, capture_faces(&fresh)?, incidence)?;
    if recaptured.replay_bytes() != projection.replay_bytes() {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "full geometry/incidence/history/evidence changed during capture",
        });
    }
    Ok(())
}
fn check_current_contexts(bound: &BoundProjection<'_>) -> Result<(), PrismFreezeCause> {
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
