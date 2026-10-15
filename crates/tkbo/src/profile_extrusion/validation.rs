// SPDX-License-Identifier: GPL-2.0-only
//! Ordered validation of published shapes. Region faces run the shared ADR 0014 stages
//! (topology, geometry, self-intersection), then the tolerance stage against the caller cap.
//! Extruded solids run the public `body_check::validate_body` (ADR 0072, ADR 0082), so an
//! `OperationResult` carries the same `ValidationReport` an application gets for any body.
use super::decline::{
    DeclineCause, Decliner, OperationDecline, OperationStage as Stage, ValidationStageKind,
};
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
    /// The general loop region proof (ADR 0066): contacts only at authored joins, Jordan
    /// loops and certified nesting.
    RegionContactCertificate,
    /// The affine sweep `(p, t) -> p + t d` of a certified region, with `d` outside the
    /// plane by more than the angular tolerance, is injective.
    AffineSweepOfCertifiedRegion,
}

/// Ordered validation coverage of a published region face; every stage passed on success.
/// (Solids report the public `body_check::ValidationReport`, ADR 0082.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionValidation {
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

/// A shape that passed every ordered stage, with its measured tolerance and findings.
pub(crate) struct CheckedShape {
    pub validation: RegionValidation,
    pub achieved: AchievedTolerance,
    pub findings: Vec<Finding>,
}

/// Validate, measure the achieved tolerance and apply the cap, in order;
/// `check_publication(&shape, evidence, cap, decliner)?`.
pub(crate) fn check_publication(
    shape: &Shape,
    evidence: SelfIntersectionEvidence,
    cap: ToleranceCap,
    decline: Decliner,
) -> Result<CheckedShape, OperationDecline> {
    let validation =
        ordered_validation(shape, evidence).map_err(|c| decline.at(Stage::Validation, c))?;
    let achieved = achieved_tolerance(shape)
        .map_err(|f| decline.native(Stage::Tolerance, "achieved tolerance", f))?;
    let mut checked = CheckedShape {
        validation,
        achieved,
        findings: Vec::new(),
    };
    tolerance_stage(&mut checked, cap).map_err(|c| decline.at(Stage::Tolerance, c))?;
    Ok(checked)
}

/// Largest stored tolerances of `shape`; `achieved_tolerance(&shape)?`.
pub(crate) fn achieved_tolerance(shape: &Shape) -> Result<AchievedTolerance, Failure> {
    Ok(AchievedTolerance {
        vertex: BRepTool::max_tolerance(shape, ShapeEnum::Vertex)?,
        edge: BRepTool::max_tolerance(shape, ShapeEnum::Edge)?,
        face: BRepTool::max_tolerance(shape, ShapeEnum::Face)?,
    })
}

/// Run the shared ordered stages on a region face. Topology and geometry must pass; a
/// native self-intersection interference declines, and `evidence` certifies the stage.
fn ordered_validation(
    shape: &Shape,
    evidence: SelfIntersectionEvidence,
) -> Result<RegionValidation, DeclineCause> {
    let coverage = validate_owned_candidate(shape, BodyKind::PlanarRegion, &[]);
    if let Some(stage) = first_unpassed(&coverage) {
        return Err(DeclineCause::ValidationFailed {
            stage,
            findings: findings(&coverage),
        });
    }
    Ok(RegionValidation {
        topology: StageStatus::Passed,
        geometry: StageStatus::Passed,
        self_intersection: StageStatus::Passed,
        self_intersection_evidence: evidence,
        tolerance: StageStatus::Blocked,
    })
}

/// The first of topology or geometry short of Passed, else a failed self-check.
fn first_unpassed(coverage: &OrderedCoverage) -> Option<ValidationStageKind> {
    let stages = [
        (
            ValidationStageKind::Topology,
            coverage.topology != StageCoverage::Passed,
        ),
        (
            ValidationStageKind::Geometry,
            coverage.geometry != StageCoverage::Passed,
        ),
        (
            ValidationStageKind::SelfIntersection,
            coverage.self_intersection == StageCoverage::Failed,
        ),
    ];
    stages
        .into_iter()
        .find(|(_, unpassed)| *unpassed)
        .map(|(stage, _)| stage)
}

fn findings(coverage: &OrderedCoverage) -> Vec<String> {
    let render = |finding: &crate::sketch_region_validation::outcome::ValidationFinding| {
        format!(
            "{:?} {:?}: {:?}",
            finding.stage, finding.coverage, finding.cause
        )
    };
    coverage.findings.iter().map(render).collect()
}

/// The tolerance stage: refuse over a `Refuse` cap, record a finding over a `Report` cap.
fn tolerance_stage(checked: &mut CheckedShape, cap: ToleranceCap) -> Result<(), DeclineCause> {
    checked.findings = cap_findings(checked.achieved, cap)?;
    checked.validation.tolerance = StageStatus::Passed;
    Ok(())
}

/// `Refuse` over the cap declines; `Report` over the cap is one finding; else none.
pub(crate) fn cap_findings(
    achieved: AchievedTolerance,
    cap: ToleranceCap,
) -> Result<Vec<Finding>, DeclineCause> {
    let largest = achieved.max();
    match cap {
        _ if achieved.within(cap) => Ok(Vec::new()),
        ToleranceCap::Refuse(limit) => Err(DeclineCause::ToleranceOverBudget {
            achieved: largest,
            cap: limit,
        }),
        ToleranceCap::Report(limit) => Ok(vec![Finding::ToleranceAboveReportLimit {
            achieved: largest,
            limit,
        }]),
        ToleranceCap::NoLimit => Ok(Vec::new()),
    }
}
