// SPDX-License-Identifier: GPL-2.0-only
use super::{query::source_query, types::*};
use crate::sketch_region_certification::BoundaryInputs;
use rustycad_tkmath::predicates::roots::metric_bounds::certify_transported_residual;

/// Inspect all captured original→copy→basis bindings; never certifies a contact or whole region.
/// Example: `inspect_source_conformance(&inputs, policy, || false)` preserves failed work.
pub(crate) fn inspect_source_conformance(
    inputs: &BoundaryInputs,
    policy: SourceConformancePolicy,
    mut cancelled: impl FnMut() -> bool,
) -> SourceConformanceInspection {
    let mut inspection = SourceConformanceInspection {
        entries: Vec::new(),
        requested: policy,
        originals: inputs.originals.clone(),
        constructions: inputs.constructions.clone(),
        bisections: 0,
    };
    append_constructions(inputs, &mut inspection, &mut cancelled);
    inspection
}
fn append_constructions(
    inputs: &BoundaryInputs,
    inspection: &mut SourceConformanceInspection,
    cancelled: &mut impl FnMut() -> bool,
) {
    for construction in &inputs.constructions {
        for parent in &construction.parents {
            let stages = inspect_parent(inputs, construction, parent, inspection, cancelled);
            inspection.entries.push(OriginalConstructionEvidence {
                occurrence: construction.occurrence,
                original: parent.clone(),
                stages,
            });
        }
    }
}
fn inspect_parent(
    inputs: &BoundaryInputs,
    construction: &CapturedBoundaryConstruction,
    parent: &crate::sketch_region::OriginalMemberIdentity,
    inspection: &mut SourceConformanceInspection,
    cancelled: &mut impl FnMut() -> bool,
) -> Vec<ConstructionStageEvidence> {
    let original = inputs.originals.iter().find(|s| s.original == *parent);
    let mut stages = Vec::new();
    for stage in [
        ConstructionStage::OriginalPlane,
        ConstructionStage::CopiedPlane,
        ConstructionStage::CapturedBasis,
    ] {
        let proof = inspect_stage(original, construction, parent, stage, inspection, cancelled);
        let passed = matches!(
            proof.decision,
            Ok(rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision::Within { .. })
        );
        stages.push(proof);
        if !passed {
            break;
        }
    }
    stages
}
fn inspect_stage(
    original: Option<&OriginalCurveCapture>,
    construction: &CapturedBoundaryConstruction,
    parent: &crate::sketch_region::OriginalMemberIdentity,
    stage: ConstructionStage,
    inspection: &mut SourceConformanceInspection,
    cancelled: &mut impl FnMut() -> bool,
) -> ConstructionStageEvidence {
    let (limit, roots) = effective_settings(inspection, stage);
    let query = original_query(original, parent, construction, stage, limit, roots);
    let (decision, bisections) = execute_query(query, cancelled);
    inspection.bisections += bisections;
    ConstructionStageEvidence {
        stage,
        requested_limit: limit,
        effective_limit: limit,
        effective_roots: roots,
        bisections,
        decision,
    }
}
fn effective_settings(
    inspection: &SourceConformanceInspection,
    stage: ConstructionStage,
) -> (
    f64,
    rustycad_tkmath::predicates::roots::RootIsolationOptions,
) {
    let limit = if stage == ConstructionStage::CapturedBasis {
        inspection.requested.construction_limit
    } else {
        inspection.requested.original_plane_limit
    };
    let mut roots = inspection.requested.roots;
    roots.max_bisections = roots
        .max_bisections
        .saturating_sub(inspection.bisections.min(u16::MAX as u64) as u16);
    (limit, roots)
}
fn original_query<'a>(
    original: Option<&'a OriginalCurveCapture>,
    parent: &crate::sketch_region::OriginalMemberIdentity,
    construction: &'a CapturedBoundaryConstruction,
    stage: ConstructionStage,
    limit: f64,
    roots: rustycad_tkmath::predicates::roots::RootIsolationOptions,
) -> Result<
    rustycad_tkmath::predicates::roots::metric_bounds::TransportedResidualQuery<'a>,
    SourceConformanceCause,
> {
    let original =
        original.ok_or_else(|| SourceConformanceCause::MissingOriginal(parent.clone()))?;
    let original = original
        .representation
        .as_ref()
        .map_err(|e| SourceConformanceCause::Capture(e.clone()))?;
    source_query(original, construction, stage, limit, roots)
}
fn execute_query(
    query: Result<
        rustycad_tkmath::predicates::roots::metric_bounds::TransportedResidualQuery<'_>,
        SourceConformanceCause,
    >,
    cancelled: &mut impl FnMut() -> bool,
) -> (
    Result<
        rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision,
        SourceConformanceCause,
    >,
    u64,
) {
    let query = match query {
        Ok(query) => query,
        Err(cause) => return (Err(cause), 0),
    };
    let report = certify_transported_residual(query, cancelled);
    (
        report.decision.map_err(SourceConformanceCause::Exact),
        report.bisections,
    )
}
