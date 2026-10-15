// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0082: an extruded solid is validated by the public `body_check::validate_body`
//! (ADR 0072), so `OperationResult::validation()` is the same `ValidationReport`, with the
//! same stages, modes and findings, that an application gets for any body.
use super::decline::{
    DeclineCause, Decliner, OperationDecline, OperationStage as Stage, ValidationStageKind,
};
use super::resolution::{AchievedTolerance, Resolution, ToleranceCap};
use super::validation::{Finding, achieved_tolerance, cap_findings};
use crate::body_check::{
    FindingKind, StageReport, ValidationReport, ValidationStage, validate_body,
};
use rustycad_tkbrep::topods::Shape;

/// A solid whose four ordered stages passed, its achieved tolerance and cap findings.
pub(crate) struct CheckedSolid {
    pub validation: ValidationReport,
    pub achieved: AchievedTolerance,
    pub findings: Vec<Finding>,
}

/// Validate `shape` under `resolution`; any stage short of `Passed` declines.
/// Example: `check_solid(&solid, resolution, DECLINE)?`.
pub(crate) fn check_solid(
    shape: &Shape,
    resolution: Resolution,
    decline: Decliner,
) -> Result<CheckedSolid, OperationDecline> {
    let rejected = |r: crate::body_check::ResolutionRejected| {
        let (field, value) = (r.field, r.value);
        decline.at(
            Stage::Admission,
            DeclineCause::InvalidResolution { field, value },
        )
    };
    let validation = validate_body(shape, &budgeted(resolution)).map_err(rejected)?;
    let achieved = achieved_tolerance(shape)
        .map_err(|f| decline.native(Stage::Tolerance, "achieved tolerance", f))?;
    if let Some(stage) = validation.first_non_pass() {
        return Err(decline.at(stage_of(stage), non_pass(stage, achieved, resolution.cap)));
    }
    let findings =
        cap_findings(achieved, resolution.cap).map_err(|c| decline.at(Stage::Tolerance, c))?;
    Ok(CheckedSolid {
        validation,
        achieved,
        findings,
    })
}

/// Only a refusing cap is a validation budget. A report cap is a finding, not a defect
/// (ADR 0068), so the body check then runs without a budget.
fn budgeted(resolution: Resolution) -> Resolution {
    match resolution.cap {
        ToleranceCap::Refuse(_) => resolution,
        _ => Resolution {
            cap: ToleranceCap::NoLimit,
            ..resolution
        },
    }
}

/// A tolerance stage that failed only on the budget is the operation's over-budget decline.
fn only_over_budget(stage: &StageReport) -> bool {
    let over = |kind: &FindingKind| matches!(kind, FindingKind::ToleranceAboveBudget { .. });
    stage.stage == ValidationStage::Tolerance
        && stage.findings.iter().all(|finding| over(&finding.kind))
}

fn stage_of(stage: &StageReport) -> Stage {
    match only_over_budget(stage) {
        true => Stage::Tolerance,
        false => Stage::Validation,
    }
}

fn non_pass(stage: &StageReport, achieved: AchievedTolerance, cap: ToleranceCap) -> DeclineCause {
    if let (true, ToleranceCap::Refuse(limit)) = (only_over_budget(stage), cap) {
        let achieved = achieved.max();
        return DeclineCause::ToleranceOverBudget {
            achieved,
            cap: limit,
        };
    }
    let render = |finding: &crate::body_check::Finding| {
        let (severity, kind) = (finding.severity, &finding.kind);
        format!("{severity:?} {kind:?} at {:?}", finding.subject)
    };
    DeclineCause::ValidationFailed {
        stage: stage_kind(stage.stage),
        findings: stage.findings.iter().map(render).collect(),
    }
}

fn stage_kind(stage: ValidationStage) -> ValidationStageKind {
    match stage {
        ValidationStage::Topology => ValidationStageKind::Topology,
        ValidationStage::Geometry => ValidationStageKind::Geometry,
        ValidationStage::SelfIntersection => ValidationStageKind::SelfIntersection,
        ValidationStage::Tolerance => ValidationStageKind::Tolerance,
    }
}
