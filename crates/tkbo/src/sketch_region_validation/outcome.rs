// SPDX-License-Identifier: GPL-2.0-only
//! Coverage and diagnostic identities are independent of mapped validity booleans.
use crate::bop_algo::{FaceTermination, NativeAlertObservation, SelfCheckPhase};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tktopalgo::brep_check::BRepCheckStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BodyKind {
    PlanarRegion,
    Solid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ValidationStage {
    Topology,
    Geometry,
    SelfIntersection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StageCoverage {
    Passed,
    Failed,
    Incomplete,
    Blocked,
}

pub(crate) struct SourceRelation {
    pub(crate) output: Shape,
    pub(crate) parents: Vec<u64>,
}

#[derive(Debug)]
pub(crate) struct AffectedOccurrence {
    pub(crate) shape: Shape,
    // None means attribution unknown; never invent a first-parent/root relation.
    pub(crate) source_parents: Option<Vec<u64>>,
}

#[derive(Debug)]
pub(crate) enum ValidationCause {
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
        reason: AnalyticGap,
    },
    AnalyticFailure {
        phase: SelfCheckPhase,
        index: Option<i32>,
        failure: Failure,
    },
    AnalyticNativeAlert(NativeAlertObservation),
    AnalyticInterference {
        first_index: i32,
        second_index: i32,
    },
}

#[derive(Debug)]
pub(crate) enum AnalyticGap {
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

#[derive(Debug)]
pub(crate) struct ValidationFinding {
    pub(crate) stage: ValidationStage,
    pub(crate) coverage: StageCoverage,
    pub(crate) cause: ValidationCause,
    pub(crate) affected: AffectedOccurrence,
    pub(crate) context: Option<AffectedOccurrence>,
}

pub(crate) struct OrderedCoverage {
    pub(crate) topology: StageCoverage,
    pub(crate) geometry: StageCoverage,
    pub(crate) self_intersection: StageCoverage,
    pub(crate) tolerance: StageCoverage,
    pub(crate) findings: Vec<ValidationFinding>,
}

impl OrderedCoverage {
    pub(super) fn pending() -> Self {
        Self {
            topology: StageCoverage::Blocked,
            geometry: StageCoverage::Blocked,
            self_intersection: StageCoverage::Blocked,
            tolerance: StageCoverage::Blocked,
            findings: Vec::new(),
        }
    }

    pub(super) fn record(&mut self, stage: ValidationStage, findings: Vec<ValidationFinding>) {
        let coverage = summarize_findings(&findings);
        match stage {
            ValidationStage::Topology => self.topology = coverage,
            ValidationStage::Geometry => self.geometry = coverage,
            ValidationStage::SelfIntersection => self.self_intersection = coverage,
        }
        self.findings.extend(findings);
    }

    pub(crate) fn complete(&self) -> bool {
        [
            self.topology,
            self.geometry,
            self.self_intersection,
            self.tolerance,
        ]
        .iter()
        .all(|coverage| *coverage == StageCoverage::Passed)
    }
}

fn summarize_findings(findings: &[ValidationFinding]) -> StageCoverage {
    if findings
        .iter()
        .any(|finding| finding.coverage == StageCoverage::Failed)
    {
        return StageCoverage::Failed;
    }
    if !findings.is_empty() {
        return StageCoverage::Incomplete;
    }
    StageCoverage::Passed
}

pub(super) fn attributed_occurrence(
    shape: &Shape,
    sources: &[SourceRelation],
) -> AffectedOccurrence {
    let matches: Vec<&SourceRelation> = sources
        .iter()
        .filter(|entry| entry.output.is_equal(shape))
        .collect();
    let source_parents = merged_parents(&matches);
    AffectedOccurrence {
        shape: shape.clone(),
        source_parents,
    }
}

fn merged_parents(matches: &[&SourceRelation]) -> Option<Vec<u64>> {
    if matches.is_empty() {
        return None;
    }
    let mut parents: Vec<u64> = matches
        .iter()
        .flat_map(|entry| entry.parents.iter().copied())
        .collect();
    parents.sort_unstable();
    parents.dedup();
    Some(parents)
}
