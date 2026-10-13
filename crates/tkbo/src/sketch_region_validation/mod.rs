// SPDX-License-Identifier: GPL-2.0-only
//! Martensite ADR0014 partial ordered stage; no successful region/body API.
//! Validation consumes an already-owned arrangement candidate, not caller inputs.
mod body;
mod contexts;
#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod diagnostic_tests;
#[cfg(test)]
mod geometry_tests;
mod inspection;
mod outcome;
mod statuses;
#[cfg(test)]
mod tests;

pub(crate) use outcome::{
    BodyKind, OrderedCoverage, SourceRelation, StageCoverage, ValidationStage,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

/// Inspect an already-owned candidate without exposing diagnostic geometry as success.
/// Example: `validate_owned_candidate(&owned_face, BodyKind::PlanarRegion, &relations)`.
/// Passed means mapped coverage only; pending analytic certificates remain explicit.
pub(crate) fn validate_owned_candidate(
    shape: &Shape,
    kind: BodyKind,
    sources: &[SourceRelation],
) -> OrderedCoverage {
    let mut report = OrderedCoverage::pending();
    inspect_stage(shape, kind, sources, ValidationStage::Topology, &mut report);
    if report.topology != StageCoverage::Passed {
        return report;
    }
    inspect_stage(shape, kind, sources, ValidationStage::Geometry, &mut report);
    if report.geometry == StageCoverage::Passed {
        report.self_intersection = StageCoverage::Incomplete;
    }
    report
}

fn inspect_stage(
    shape: &Shape,
    kind: BodyKind,
    sources: &[SourceRelation],
    stage: ValidationStage,
    report: &mut OrderedCoverage,
) {
    let outcome = BRepCheckAnalyzer::new(shape, stage == ValidationStage::Geometry, false, true);
    let findings = match outcome {
        Ok(analyzer) => inspection::inspect_candidate(&analyzer, shape, kind, sources, stage),
        Err(failure) => vec![inspection::native_failure(shape, sources, stage, failure)],
    };
    report.record(stage, findings);
}
