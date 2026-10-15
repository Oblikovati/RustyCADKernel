// SPDX-License-Identifier: GPL-2.0-only
//! The mapped checker stages of ADR 0014, reused unchanged for any body and translated
//! into public findings. The two structural self-check gaps (inner search and pair
//! completion are never observable natively) become the stage's `NativeSearch` mode
//! instead of findings; every other gap stays inconclusive.
use super::naming::TopologyNames;
use super::report::{Finding, FindingKind, Severity};
use crate::sketch_region_validation::{
    AnalyticGap, BodyKind, OrderedCoverage, StageCoverage, ValidationCause, ValidationFinding,
    ValidationStage, inspect_self_intersection, inspect_stage,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The mapped BRepCheck findings of the topology or geometry stage.
/// Example: `checker_findings(&solid, ValidationStage::Geometry, &names)`.
pub(super) fn checker_findings(
    shape: &Shape,
    stage: ValidationStage,
    names: &TopologyNames,
) -> Vec<Finding> {
    let mut coverage = OrderedCoverage::pending();
    inspect_stage(shape, BodyKind::Any, &[], stage, &mut coverage);
    translate(&coverage.findings, names)
}

/// The mapped BOPAlgo_CheckerSI findings; `self_intersection_findings(&solid, &names)`.
pub(super) fn self_intersection_findings(shape: &Shape, names: &TopologyNames) -> Vec<Finding> {
    translate(&inspect_self_intersection(shape, &[]), names)
}

fn translate(findings: &[ValidationFinding], names: &TopologyNames) -> Vec<Finding> {
    let checked = findings
        .iter()
        .filter(|finding| !container_without_checker(finding));
    checked
        .flat_map(|finding| translate_one(finding, names))
        .collect()
}

/// One public finding per kind of `finding`'s cause, naming its occurrence and context.
fn translate_one(finding: &ValidationFinding, names: &TopologyNames) -> Vec<Finding> {
    let severity = match finding.coverage {
        StageCoverage::Failed => Severity::Defect,
        _ => Severity::Inconclusive,
    };
    let subject = names.name(&finding.affected.shape);
    let context = finding
        .context
        .as_ref()
        .map(|context| names.name(&context.shape));
    let kinds = kinds(&finding.cause).into_iter();
    kinds
        .map(|kind| Finding {
            severity,
            kind,
            subject,
            context,
        })
        .collect()
}

/// `BRepCheck_Analyzer::Put` creates no result for compounds and comp-solids
/// (`case TopAbs_COMPSOLID: case TopAbs_COMPOUND: break;`), so their missing checker is
/// not a coverage gap: their members are checked individually.
fn container_without_checker(finding: &ValidationFinding) -> bool {
    let container = matches!(
        finding.affected.shape.shape_type(),
        Ok(ShapeEnum::Compound | ShapeEnum::CompSolid)
    );
    container && matches!(finding.cause, ValidationCause::MissingChecker)
}

fn kinds(cause: &ValidationCause) -> Vec<FindingKind> {
    match cause {
        ValidationCause::BodyKindMismatch { actual, .. } => {
            vec![FindingKind::UnsupportedShape { actual: *actual }]
        }
        ValidationCause::NativeStatuses(statuses) => statuses
            .iter()
            .map(|status| FindingKind::CheckStatus(*status))
            .collect(),
        ValidationCause::NativeFailure(failure) => vec![FindingKind::CheckerFailure {
            message: failure.message().to_owned(),
        }],
        other => analytic_kind(other).into_iter().collect(),
    }
}

/// The self-intersection causes; the remaining mapped causes are coverage gaps.
fn analytic_kind(cause: &ValidationCause) -> Option<FindingKind> {
    Some(match cause {
        ValidationCause::AnalyticGap { reason, .. } => return gap_kind(reason),
        ValidationCause::AnalyticFailure { phase, failure, .. } => {
            let message = format!("{phase:?}: {}", failure.message());
            FindingKind::CheckerFailure { message }
        }
        ValidationCause::AnalyticNativeAlert(alert) => FindingKind::SearchIncomplete {
            reason: format!("{alert:?}"),
        },
        ValidationCause::AnalyticInterference { .. } => FindingKind::Interference,
        _ => FindingKind::CheckerCoverageGap,
    })
}

/// None for the gaps that define the native search mode rather than a failed run.
fn gap_kind(reason: &AnalyticGap) -> Option<FindingKind> {
    match reason {
        AnalyticGap::InnerSearchUnobserved | AnalyticGap::PairCompletionUnobserved => None,
        other => Some(FindingKind::SearchIncomplete {
            reason: format!("{other:?}"),
        }),
    }
}
