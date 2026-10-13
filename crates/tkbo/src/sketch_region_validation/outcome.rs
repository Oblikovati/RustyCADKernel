// SPDX-License-Identifier: GPL-2.0-only
//! Coverage and diagnostic identities are independent of mapped validity booleans.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
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
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StageCoverage {
    Passed,
    Failed,
    Incomplete,
    Blocked,
}

use super::source_relations::SourceMemberRelations;
pub(crate) use super::source_relations::SourceRelation;
pub(super) use super::source_relations::attributed_occurrence;

#[derive(Debug)]
pub(crate) struct AffectedOccurrence {
    pub(crate) shape: Shape,
    // None means attribution unknown; never invent a first-parent/root relation.
    pub(crate) original_parents: Option<SourceMemberRelations>,
    // Derived caller IDs cannot replace the complete member-and-kind attribution.
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
