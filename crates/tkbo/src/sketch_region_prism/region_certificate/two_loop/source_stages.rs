// SPDX-License-Identifier: GPL-2.0-only
//! Revalidate the existing full source matrix and retain its exact zero-residual stages.
use super::{failure::*, outcome::TwoLoopSourceStage, work::*};
use crate::sketch_region_certification::BoundaryCensus;
use crate::sketch_region_source_conformance::*;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

pub(super) fn current<'a>(
    census: &'a BoundaryCensus,
    work: &TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, &'a SourceConformanceInspection> {
    let bound = census.source_binding.as_ref().ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "retained source conformance",
        })
    })?;
    let policy = census.source_policy.ok_or_else(|| {
        work.failure(TwoLoopRegionCause::Missing {
            hypothesis: "requested source policy",
        })
    })?;
    let mut faults = source_binding_faults(&census.inputs, policy, bound.inspection());
    faults.extend_from_slice(bound.faults());
    if !faults.is_empty() {
        let mut failure = work.failure(TwoLoopRegionCause::UnsupportedDiagnostic {
            stage: "actual source binding faults",
        });
        failure.local = Some(TwoLoopLocalDiagnostic::SourceBinding(faults));
        return Err(failure);
    }
    Ok(bound.inspection())
}
pub(super) fn stages<'a>(
    entry: &'a OriginalConstructionEvidence,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, Vec<TwoLoopSourceStage>> {
    use ConstructionStage::*;
    work.count("complete source stage roster", entry.stages.len(), 3)?;
    for stage in [OriginalPlane, CopiedPlane, CapturedBasis] {
        work.count(
            "one actual source stage",
            entry.stages.iter().filter(|r| r.stage == stage).count(),
            1,
        )?;
    }
    let mut result = Vec::new();
    for stage in &entry.stages {
        result.push(stage_row(stage, work)?);
    }
    Ok(result)
}
fn stage_row<'a>(
    row: &'a ConstructionStageEvidence,
    work: &mut TwoLoopWork<'_>,
) -> TwoLoopCheck<'a, TwoLoopSourceStage> {
    let residual = row.decision.as_ref().map_err(|cause| {
        let mut failure = work.failure(TwoLoopRegionCause::SourceRefusal { stage: row.stage });
        failure.local = Some(TwoLoopLocalDiagnostic::Source(cause));
        failure
    })?;
    work.holds(
        "zero exact source conformance",
        matches!(residual, ResidualDecision::Within { proven_squared_upper }
        if proven_squared_upper.compare_f64(0.).is_ok_and(|order| order.is_eq())),
    )?;
    work.units("source conformance stage", 1)?;
    Ok(TwoLoopSourceStage {
        stage: row.stage,
        requested_limit: row.requested_limit,
        effective_limit: row.effective_limit,
        effective_roots: row.effective_roots,
        bisections: row.bisections,
        residual: residual.clone(),
    })
}
