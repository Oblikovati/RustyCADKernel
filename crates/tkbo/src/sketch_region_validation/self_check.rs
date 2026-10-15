// SPDX-License-Identifier: GPL-2.0-only
//! Outer termination and inner search evidence must remain separate.
use super::outcome::StageCoverage;
use rustycad_tkgeomalgo::int_walk::IntWalkPWalking;

pub(super) struct WalkingObservation {
    pub(super) done: bool,
    pub(super) start: [f64; 4],
}

pub(super) fn observe_walking(walk: &IntWalkPWalking, start: [f64; 4]) -> WalkingObservation {
    WalkingObservation {
        done: walk.is_done(),
        start,
    }
}

pub(super) fn walking_coverage(witness: &WalkingObservation) -> StageCoverage {
    if !witness.done {
        return StageCoverage::Incomplete;
    }
    // This covers one actual walking execution, never all seeds or pruning.
    StageCoverage::Passed
}

use super::outcome::{
    AffectedOccurrence, AnalyticGap, SourceRelation, ValidationCause, ValidationFinding,
    ValidationStage, attributed_occurrence,
};
use crate::bop_algo::{BOPAlgoCheckerSI, FaceTermination, SelfCheckCompletion};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{ncollection::List, standard::Failure};

pub(crate) fn inspect_self_intersection(
    shape: &Shape,
    sources: &[SourceRelation],
) -> Vec<ValidationFinding> {
    let mut checker = BOPAlgoCheckerSI::new();
    checker.set_arguments(&[shape.clone()].into_iter().collect::<List<Shape>>());
    checker.options_mut().set_run_parallel(false);
    let trace = checker.perform_observed();
    let mut inspection = AnalyticInspection {
        checker: &checker,
        root: shape,
        sources,
        findings: Vec::new(),
    };
    inspection.consume(trace);
    inspection.findings
}

struct AnalyticInspection<'a> {
    checker: &'a BOPAlgoCheckerSI,
    root: &'a Shape,
    sources: &'a [SourceRelation],
    findings: Vec<ValidationFinding>,
}

impl AnalyticInspection<'_> {
    fn consume(&mut self, mut trace: SelfCheckCompletion) {
        self.captured_failure(&mut trace);
        for alert in trace.alerts.drain(..) {
            self.push(
                None,
                None,
                StageCoverage::Incomplete,
                ValidationCause::AnalyticNativeAlert(alert),
            );
        }
        self.run_gaps(&trace);
        self.face_gaps(&trace);
        self.native_interferences();
        self.gap(None, None, None, AnalyticGap::PairCompletionUnobserved);
    }

    fn captured_failure(&mut self, trace: &mut SelfCheckCompletion) {
        if let Some(failure) = trace.failure.take() {
            let cause = ValidationCause::AnalyticFailure {
                phase: trace.phase,
                index: trace.active_source,
                failure,
            };
            self.push(trace.active_source, None, StageCoverage::Incomplete, cause);
        }
    }

    fn run_gaps(&mut self, trace: &SelfCheckCompletion) {
        self.rejected_arguments(trace);
        if !trace.returned {
            self.gap(None, None, None, AnalyticGap::NativeRunIncomplete);
        }
        if trace.native_alerts {
            self.gap(None, None, None, AnalyticGap::NativeAlerts);
        }
        if trace.level < 9 {
            self.gap(None, None, None, AnalyticGap::LevelNotCovered(trace.level));
        }
        if !trace.face_phase_scheduled {
            self.gap(None, None, None, AnalyticGap::FacePhaseUnscheduled);
        }
    }

    fn rejected_arguments(&mut self, trace: &SelfCheckCompletion) {
        if let Some(actual) = trace.rejected_arguments {
            self.gap(
                None,
                None,
                None,
                AnalyticGap::ArgumentsRejected {
                    actual,
                    expected: 1,
                },
            );
        }
    }

    fn face_gaps(&mut self, trace: &SelfCheckCompletion) {
        for face in &trace.faces {
            let reason = match face.termination {
                FaceTermination::PolicyExcluded => continue,
                FaceTermination::NotAttempted => AnalyticGap::FaceNotAttempted,
                FaceTermination::NotDone => AnalyticGap::OuterNotDone,
                FaceTermination::Terminated { .. } => AnalyticGap::InnerSearchUnobserved,
            };
            self.gap(
                Some(face.source_index),
                Some(face.kind),
                Some(face.termination),
                reason,
            );
        }
    }

    fn native_interferences(&mut self) {
        let Some(structure) = self.checker.data_structure() else {
            return;
        };
        for pair in structure.interferences().iter() {
            let (first_index, second_index) = pair.indices();
            let cause = ValidationCause::AnalyticInterference {
                first_index,
                second_index,
            };
            self.push(
                Some(first_index),
                Some(second_index),
                StageCoverage::Failed,
                cause,
            );
        }
    }

    fn gap(
        &mut self,
        index: Option<i32>,
        kind: Option<rustycad_tkmath::geom_abs::SurfaceType>,
        outer: Option<FaceTermination>,
        reason: AnalyticGap,
    ) {
        let cause = ValidationCause::AnalyticGap {
            index,
            kind,
            outer,
            reason,
        };
        self.push(index, None, StageCoverage::Incomplete, cause);
    }

    fn push(
        &mut self,
        index: Option<i32>,
        second: Option<i32>,
        coverage: StageCoverage,
        cause: ValidationCause,
    ) {
        let resolved = self.resolve(index, second);
        self.push_resolution(resolved, coverage, cause);
    }

    fn push_resolution(
        &mut self,
        resolved: Result<ResolvedWitness, Failure>,
        coverage: StageCoverage,
        cause: ValidationCause,
    ) {
        match resolved {
            Ok((affected, context)) => self.findings.push(ValidationFinding {
                stage: ValidationStage::SelfIntersection,
                coverage,
                cause,
                affected,
                context,
            }),
            Err(failure) => self.push_native_failure(failure),
        }
    }

    fn push_native_failure(&mut self, failure: Failure) {
        let finding = super::inspection::native_failure(
            self.root,
            self.sources,
            ValidationStage::SelfIntersection,
            failure,
        );
        self.findings.push(finding);
    }

    fn resolve(&self, index: Option<i32>, second: Option<i32>) -> Result<ResolvedWitness, Failure> {
        let affected = attributed_occurrence(self.indexed_shape(index)?, self.sources);
        let context = second
            .map(|index| {
                self.indexed_shape(Some(index))
                    .map(|shape| attributed_occurrence(shape, self.sources))
            })
            .transpose()?;
        Ok((affected, context))
    }

    fn indexed_shape(&self, index: Option<i32>) -> Result<&Shape, Failure> {
        let Some(index) = index else {
            return Ok(self.root);
        };
        let structure = self.checker.data_structure().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "Self-check source index has no data structure; expected original run DS",
                "",
            )
        })?;
        Ok(structure.shape_info(index)?.shape())
    }
}

type ResolvedWitness = (AffectedOccurrence, Option<AffectedOccurrence>);
