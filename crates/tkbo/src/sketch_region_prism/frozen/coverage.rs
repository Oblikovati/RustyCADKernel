// SPDX-License-Identifier: GPL-2.0-only
//! Mapped validation is retained as unbound history, never promoted to frozen body validity.
use super::{
    bindings::{BoundProjection, missing},
    history::{occurrence_keys, original_keys},
    types::*,
};
use crate::bop_algo::{FaceTermination, SelfCheckPhase};
use crate::sketch_region_validation::outcome::*;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tktopalgo::brep_check::BRepCheckStatus;

#[derive(Clone, Debug)]
pub(crate) struct FrozenPrismCoverage {
    pub(crate) current: [StageCoverage; 4],
    pub(crate) historical_unbound: [StageCoverage; 4],
    pub(crate) findings: Vec<FrozenValidationFinding>,
    pub(crate) limitations: Vec<FrozenPublicationGap>,
}
#[derive(Clone, Debug)]
pub(crate) enum FrozenPublicationGap {
    WholeBodyValidationIncomplete,
    HistoricalValidationUnbound,
    SelectionEvidenceNotImported,
    RejectedSelectionHistoryUnbound,
    AffineEvidenceNotImported,
    IntermediateRetirementCompositionUnproven,
    Candidate(crate::sketch_region_candidates::MissingCertificate),
    PrismInjectivityAndSurfaceCharts,
    AchievedErrorUnknown,
}
#[derive(Clone, Debug)]
pub(crate) struct FrozenValidationFinding {
    pub(crate) stage: ValidationStage,
    pub(crate) coverage: StageCoverage,
    pub(crate) cause: FrozenValidationCause,
    pub(crate) affected: FrozenAffectedOccurrence,
    pub(crate) context: Option<FrozenAffectedOccurrence>,
}
#[derive(Clone, Debug)]
pub(crate) struct FrozenAffectedOccurrence {
    pub(crate) occurrences: Vec<FrozenUseKey>,
    pub(crate) original_parents: [Vec<FrozenOriginalKey>; 3],
    pub(crate) source_parents: Option<Vec<u64>>,
}
#[derive(Clone, Debug)]
pub(crate) enum FrozenValidationCause {
    BodyKindMismatch {
        expected: BodyKind,
        actual: ShapeEnum,
    },
    NonPlanarRegion,
    MissingContext,
    MissingChecker,
    EmptyStatuses,
    NativeStatuses(Vec<BRepCheckStatus>),
    NativeFailure(Failure),
    AnalyticGap {
        index: Option<i32>,
        kind: Option<SurfaceType>,
        outer: Option<FaceTermination>,
        reason: FrozenAnalyticGap,
    },
    AnalyticFailure {
        phase: SelfCheckPhase,
        index: Option<i32>,
        failure: Failure,
    },
    AnalyticNativeAlert(super::alerts::FrozenNativeAlert),
    AnalyticInterference {
        first_index: i32,
        second_index: i32,
    },
}
#[derive(Clone, Debug)]
pub(crate) enum FrozenAnalyticGap {
    LevelNotCovered(i32),
    NativeRunIncomplete,
    ArgumentsRejected { actual: i32, expected: i32 },
    FacePhaseUnscheduled,
    NativeAlerts,
    FaceNotAttempted,
    OuterNotDone,
    InnerSearchUnobserved,
    PairCompletionUnobserved,
}
pub(super) fn capture_coverage(bound: &BoundProjection<'_>) -> CaptureResult<FrozenPrismCoverage> {
    let coverage = bound
        .diagnostic
        .coverage
        .as_ref()
        .ok_or(missing("historical ordered coverage"))?;
    let findings = coverage
        .findings
        .iter()
        .map(|f| freeze_finding(bound, f))
        .collect::<Result<_, _>>()?;
    Ok(FrozenPrismCoverage {
        current: [StageCoverage::Incomplete; 4],
        historical_unbound: historical_stages(coverage),
        findings,
        limitations: capture_limitations(bound)?,
    })
}
fn historical_stages(coverage: &OrderedCoverage) -> [StageCoverage; 4] {
    [
        coverage.topology,
        coverage.geometry,
        coverage.self_intersection,
        coverage.tolerance,
    ]
}

fn capture_limitations(bound: &BoundProjection<'_>) -> CaptureResult<Vec<FrozenPublicationGap>> {
    let mut limitations = vec![
        FrozenPublicationGap::WholeBodyValidationIncomplete,
        FrozenPublicationGap::HistoricalValidationUnbound,
        FrozenPublicationGap::SelectionEvidenceNotImported,
        FrozenPublicationGap::RejectedSelectionHistoryUnbound,
        FrozenPublicationGap::AffineEvidenceNotImported,
        FrozenPublicationGap::IntermediateRetirementCompositionUnproven,
    ];
    for gap in &bound.receipt.gaps {
        limitations.push(freeze_gap(gap)?);
    }
    Ok(limitations)
}

fn freeze_finding(
    bound: &BoundProjection<'_>,
    finding: &ValidationFinding,
) -> CaptureResult<FrozenValidationFinding> {
    Ok(FrozenValidationFinding {
        stage: finding.stage,
        coverage: finding.coverage,
        cause: freeze_cause(&finding.cause)?,
        affected: freeze_affected(bound, &finding.affected)?,
        context: finding
            .context
            .as_ref()
            .map(|c| freeze_affected(bound, c))
            .transpose()?,
    })
}
fn freeze_affected(
    bound: &BoundProjection<'_>,
    affected: &AffectedOccurrence,
) -> CaptureResult<FrozenAffectedOccurrence> {
    let parents = affected
        .original_parents
        .as_ref()
        .ok_or(missing("typed diagnostic original-parent attribution"))?;
    Ok(FrozenAffectedOccurrence {
        occurrences: occurrence_keys(bound, &affected.shape)?,
        original_parents: [
            original_keys(bound.bindings, &parents.preserved)?,
            original_keys(bound.bindings, &parents.modified)?,
            original_keys(bound.bindings, &parents.generated)?,
        ],
        source_parents: affected.source_parents.clone(),
    })
}
fn freeze_cause(cause: &ValidationCause) -> CaptureResult<FrozenValidationCause> {
    use FrozenValidationCause as F;
    use ValidationCause as C;
    Ok(match cause {
        C::BodyKindMismatch { expected, actual } => F::BodyKindMismatch {
            expected: *expected,
            actual: *actual,
        },
        C::NonPlanarRegion => F::NonPlanarRegion,
        C::MissingContext => F::MissingContext,
        C::MissingChecker => F::MissingChecker,
        C::EmptyStatuses => F::EmptyStatuses,
        C::NativeStatuses(s) => F::NativeStatuses(s.clone()),
        C::NativeFailure(f) => F::NativeFailure(f.clone()),
        _ => freeze_analytic_cause(cause)?,
    })
}
fn freeze_analytic_cause(cause: &ValidationCause) -> CaptureResult<FrozenValidationCause> {
    use ValidationCause as C;
    Ok(match cause {
        C::AnalyticGap {
            index,
            kind,
            outer,
            reason,
        } => analytic_gap(*index, *kind, *outer, reason),
        C::AnalyticFailure {
            phase,
            index,
            failure,
        } => analytic_failure(*phase, *index, failure),
        _ => freeze_analytic_observation(cause)?,
    })
}
fn freeze_analytic_observation(cause: &ValidationCause) -> CaptureResult<FrozenValidationCause> {
    use FrozenValidationCause as F;
    use ValidationCause as C;
    Ok(match cause {
        C::AnalyticInterference {
            first_index,
            second_index,
        } => analytic_interference(*first_index, *second_index),
        C::AnalyticNativeAlert(alert) => {
            F::AnalyticNativeAlert(super::alerts::freeze_alert(alert)?)
        }
        _ => {
            return Err(super::admission::unsupported(
                "expected analytic validation cause",
            ));
        }
    })
}

fn analytic_gap(
    index: Option<i32>,
    kind: Option<SurfaceType>,
    outer: Option<FaceTermination>,
    reason: &AnalyticGap,
) -> FrozenValidationCause {
    FrozenValidationCause::AnalyticGap {
        index,
        kind,
        outer,
        reason: freeze_analytic(reason),
    }
}
fn analytic_failure(
    phase: SelfCheckPhase,
    index: Option<i32>,
    failure: &Failure,
) -> FrozenValidationCause {
    FrozenValidationCause::AnalyticFailure {
        phase,
        index,
        failure: failure.clone(),
    }
}
fn analytic_interference(first_index: i32, second_index: i32) -> FrozenValidationCause {
    FrozenValidationCause::AnalyticInterference {
        first_index,
        second_index,
    }
}

fn freeze_analytic(reason: &AnalyticGap) -> FrozenAnalyticGap {
    use AnalyticGap as A;
    use FrozenAnalyticGap as F;
    match reason {
        A::LevelNotCovered(n) => F::LevelNotCovered(*n),
        A::NativeRunIncomplete => F::NativeRunIncomplete,
        A::ArgumentsRejected { actual, expected } => F::ArgumentsRejected {
            actual: *actual,
            expected: *expected,
        },
        A::FacePhaseUnscheduled => F::FacePhaseUnscheduled,
        A::NativeAlerts => F::NativeAlerts,
        A::FaceNotAttempted => F::FaceNotAttempted,
        A::OuterNotDone => F::OuterNotDone,
        A::InnerSearchUnobserved => F::InnerSearchUnobserved,
        A::PairCompletionUnobserved => F::PairCompletionUnobserved,
    }
}
fn freeze_gap(gap: &super::super::PrismGap) -> CaptureResult<FrozenPublicationGap> {
    use super::super::PrismGap as G;
    match gap {
        G::Candidate(c) => Ok(FrozenPublicationGap::Candidate(*c)),
        G::PrismInjectivityAndSurfaceCharts => {
            Ok(FrozenPublicationGap::PrismInjectivityAndSurfaceCharts)
        }
        G::AchievedErrorUnknown => Ok(FrozenPublicationGap::AchievedErrorUnknown),
        G::ComposedDeletionCoverage => {
            Ok(FrozenPublicationGap::IntermediateRetirementCompositionUnproven)
        }
        _ => Err(PrismFreezeCause::UnsupportedDiagnostic {
            stage: None,
            actual: "unresolved attributed transport/history gap",
        }),
    }
}
