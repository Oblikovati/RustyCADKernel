// SPDX-License-Identifier: GPL-2.0-only
//! Ordered validation through the shared ADR 0014 stages (topology, geometry,
//! self-intersection), then the tolerance stage against the caller cap.
use super::decline::{DeclineCause, ValidationStageKind};
use super::resolution::{AchievedTolerance, ToleranceCap};
use crate::sketch_region_validation::{
    BodyKind, OrderedCoverage, StageCoverage, validate_owned_candidate,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Outcome of one ordered stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageStatus {
    /// The stage ran completely and found nothing.
    Passed,
    /// The stage found a defect.
    Failed,
    /// The stage did not complete; never treated as passed.
    Incomplete,
    /// The stage did not run because an earlier stage did not pass.
    Blocked,
}

/// Why the self-intersection stage passed.
///
/// The shared native self-check (BOPAlgo_CheckerSI) still runs and any interference it
/// finds declines, but by design it never certifies completeness; the pass rests on these
/// certificates instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfIntersectionEvidence {
    /// Certified pairwise curve contacts only at authored joins, and certified loop nesting.
    RegionContactCertificate,
    /// The affine sweep `(p, t) -> p + t d` of a certified region, with `d` outside the
    /// plane by more than the angular tolerance, is injective.
    AffineSweepOfCertifiedRegion,
}

/// Ordered validation coverage of a published shape; every stage passed on success.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidationReport {
    /// Native topology checks.
    pub topology: StageStatus,
    /// Native geometry checks.
    pub geometry: StageStatus,
    /// Self-intersection stage.
    pub self_intersection: StageStatus,
    /// The certificate behind the self-intersection pass.
    pub self_intersection_evidence: SelfIntersectionEvidence,
    /// Achieved tolerance against the cap.
    pub tolerance: StageStatus,
}

/// A non-fatal observation recorded with a successful result.
#[derive(Clone, Debug, PartialEq)]
pub enum Finding {
    /// The achieved tolerance exceeds a `ToleranceCap::Report` limit.
    ToleranceAboveReportLimit {
        /// Largest stored tolerance.
        achieved: f64,
        /// The report limit.
        limit: f64,
    },
}

/// Largest stored tolerances of `shape`; `achieved_tolerance(&shape)?`.
pub(crate) fn achieved_tolerance(shape: &Shape) -> Result<AchievedTolerance, Failure> {
    Ok(AchievedTolerance {
        vertex: BRepTool::max_tolerance(shape, ShapeEnum::Vertex)?,
        edge: BRepTool::max_tolerance(shape, ShapeEnum::Edge)?,
        face: BRepTool::max_tolerance(shape, ShapeEnum::Face)?,
    })
}

/// Run the shared ordered stages. Topology and geometry must pass; a native
/// self-intersection interference declines, and `evidence` certifies the stage otherwise.
pub(crate) fn ordered_validation(
    shape: &Shape,
    evidence: SelfIntersectionEvidence,
) -> Result<ValidationReport, DeclineCause> {
    let kind = match evidence {
        SelfIntersectionEvidence::RegionContactCertificate => BodyKind::PlanarRegion,
        SelfIntersectionEvidence::AffineSweepOfCertifiedRegion => BodyKind::Solid,
    };
    let coverage = validate_owned_candidate(shape, kind, &[]);
    let stages = [
        (ValidationStageKind::Topology, coverage.topology),
        (ValidationStageKind::Geometry, coverage.geometry),
    ];
    let failed = stages
        .iter()
        .find(|(_, status)| *status != StageCoverage::Passed);
    let interference = (coverage.self_intersection == StageCoverage::Failed).then_some((
        ValidationStageKind::SelfIntersection,
        coverage.self_intersection,
    ));
    if let Some((stage, _)) = failed.copied().or(interference) {
        return Err(DeclineCause::ValidationFailed {
            stage,
            findings: findings(&coverage),
        });
    }
    Ok(ValidationReport {
        topology: StageStatus::Passed,
        geometry: StageStatus::Passed,
        self_intersection: StageStatus::Passed,
        self_intersection_evidence: evidence,
        tolerance: StageStatus::Blocked,
    })
}

fn findings(coverage: &OrderedCoverage) -> Vec<String> {
    coverage
        .findings
        .iter()
        .map(|finding| {
            format!(
                "{:?} {:?}: {:?}",
                finding.stage, finding.coverage, finding.cause
            )
        })
        .collect()
}

/// The tolerance stage: refuse over a `Refuse` cap, record a finding over a `Report` cap.
pub(crate) fn tolerance_stage(
    achieved: AchievedTolerance,
    cap: ToleranceCap,
    report: &mut ValidationReport,
    notes: &mut Vec<Finding>,
) -> Result<(), DeclineCause> {
    match cap {
        ToleranceCap::Refuse(limit) if !achieved.within(cap) => {
            return Err(DeclineCause::ToleranceOverBudget {
                achieved: achieved.max(),
                cap: limit,
            });
        }
        ToleranceCap::Report(limit) if !achieved.within(cap) => {
            notes.push(Finding::ToleranceAboveReportLimit {
                achieved: achieved.max(),
                limit,
            });
        }
        _ => {}
    }
    report.tolerance = StageStatus::Passed;
    Ok(())
}
