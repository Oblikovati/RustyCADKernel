// SPDX-License-Identifier: GPL-2.0-only
//! Preserve mapped statuses and failures without replacing native validity semantics.
use super::body::body_cause;
use super::contexts::{checked_occurrences, required_contexts};
use super::outcome::{
    BodyKind, SourceRelation, StageCoverage, ValidationCause, ValidationFinding, ValidationStage,
    attributed_occurrence,
};
use super::statuses::status_cause;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_check::{BRepCheckAnalyzer, ListOfStatus};

struct ContextInspection<'a> {
    analyzer: &'a BRepCheckAnalyzer,
    sources: &'a [SourceRelation],
    stage: ValidationStage,
    findings: Vec<ValidationFinding>,
}

pub(super) fn inspect_candidate(
    analyzer: &BRepCheckAnalyzer,
    shape: &Shape,
    kind: BodyKind,
    sources: &[SourceRelation],
    stage: ValidationStage,
) -> Vec<ValidationFinding> {
    let mut inspection = ContextInspection {
        analyzer,
        sources,
        stage,
        findings: Vec::new(),
    };
    if let Err(failure) = inspection.body(shape, kind) {
        inspection
            .findings
            .push(native_failure(shape, sources, stage, failure));
    }
    inspection.findings
}

impl ContextInspection<'_> {
    fn body(&mut self, shape: &Shape, kind: BodyKind) -> Result<(), Failure> {
        if let Some(cause) = body_cause(shape, kind, self.stage)? {
            self.push(shape, None, StageCoverage::Failed, cause);
            return Ok(());
        }
        for occurrence in checked_occurrences(shape)? {
            self.capture_occurrence(&occurrence);
        }
        Ok(())
    }

    fn capture_occurrence(&mut self, shape: &Shape) {
        if let Err(failure) = self.occurrence(shape) {
            self.push(
                shape,
                None,
                StageCoverage::Incomplete,
                ValidationCause::NativeFailure(failure),
            );
        }
    }

    fn capture_context(&mut self, subshape: &Shape, context: &Shape) {
        if let Err(failure) = self.context(subshape, context) {
            self.push(
                subshape,
                Some(context),
                StageCoverage::Incomplete,
                ValidationCause::NativeFailure(failure),
            );
        }
    }

    fn occurrence(&mut self, shape: &Shape) -> Result<(), Failure> {
        let checker = self.analyzer.result(shape)?;
        let Some(checker) = checker.get() else {
            self.push(
                shape,
                None,
                StageCoverage::Incomplete,
                ValidationCause::MissingChecker,
            );
            return Ok(());
        };
        self.statuses(shape, None, checker.status()?);
        for subshape in required_contexts(shape)? {
            self.capture_context(&subshape, shape);
        }
        Ok(())
    }

    fn context(&mut self, subshape: &Shape, context: &Shape) -> Result<(), Failure> {
        let checker = self.analyzer.result(subshape)?;
        if checker.is_null() || !checker.is_status_on_shape(context) {
            self.push(
                subshape,
                Some(context),
                StageCoverage::Incomplete,
                ValidationCause::MissingContext,
            );
            return Ok(());
        }
        self.statuses(
            subshape,
            Some(context),
            checker.status_on_shape_of(context)?,
        );
        Ok(())
    }

    fn statuses(&mut self, shape: &Shape, context: Option<&Shape>, statuses: ListOfStatus) {
        if let Some((coverage, cause)) = status_cause(statuses) {
            self.push(shape, context, coverage, cause);
        }
    }

    fn push(
        &mut self,
        shape: &Shape,
        context: Option<&Shape>,
        coverage: StageCoverage,
        cause: ValidationCause,
    ) {
        self.findings.push(ValidationFinding {
            stage: self.stage,
            coverage,
            cause,
            affected: attributed_occurrence(shape, self.sources),
            context: context.map(|shape| attributed_occurrence(shape, self.sources)),
        });
    }
}

pub(super) fn native_failure(
    shape: &Shape,
    sources: &[SourceRelation],
    stage: ValidationStage,
    failure: Failure,
) -> ValidationFinding {
    ValidationFinding {
        stage,
        coverage: StageCoverage::Incomplete,
        cause: ValidationCause::NativeFailure(failure),
        affected: attributed_occurrence(shape, sources),
        context: None,
    }
}
