// SPDX-License-Identifier: GPL-2.0-only
//! The ordered stage ledger: verdicts from findings, later stages `NotRun` after a non-pass.
use super::report::{
    CheckMode, EulerCensus, Finding, FindingKind, Severity, StageReport, StageVerdict, TopologyRef,
    ValidationStage,
};
use rustycad_tkernel::standard::Failure;

/// Ordered validation of one body: four stage reports in stage order.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationReport {
    /// Topology, geometry, self-intersection and tolerance, in this order.
    pub stages: [StageReport; 4],
    /// One Euler-Poincare census per solid whose shells passed the edge-use checks.
    pub census: Vec<EulerCensus>,
    /// The tolerance cap applied by the tolerance stage, if the resolution sets one.
    pub tolerance_budget: Option<f64>,
}

impl ValidationReport {
    /// Whether every stage passed (a sampled or searched pass included).
    /// Example: `assert!(validate_body(&solid, &resolution)?.is_valid())`.
    pub fn is_valid(&self) -> bool {
        self.stages
            .iter()
            .all(|stage| stage.verdict == StageVerdict::Passed)
    }

    /// Whether every stage passed with exact checks only; never true while the
    /// self-intersection stage rests on the native search.
    pub fn is_exact(&self) -> bool {
        self.is_valid()
            && self
                .stages
                .iter()
                .all(|stage| stage.mode == CheckMode::Exact)
    }

    /// The report of `stage`; `report.stage(ValidationStage::Tolerance).verdict`.
    pub fn stage(&self, stage: ValidationStage) -> &StageReport {
        &self.stages[stage as usize]
    }

    /// The first stage that did not pass, if any.
    pub fn first_non_pass(&self) -> Option<&StageReport> {
        let ran = self
            .stages
            .iter()
            .filter(|stage| stage.verdict != StageVerdict::NotRun);
        ran.into_iter()
            .find(|stage| stage.verdict != StageVerdict::Passed)
    }
}

/// Collects stage reports in order and blocks the remainder after a non-pass.
pub(super) struct Ledger {
    stages: Vec<StageReport>,
}

impl Ledger {
    /// An empty ledger; `Ledger::new()`.
    pub(super) fn new() -> Self {
        Self {
            stages: Vec::with_capacity(4),
        }
    }

    /// Records `stage` from its findings; returns whether later stages may run.
    pub(super) fn record(
        &mut self,
        stage: ValidationStage,
        mode: CheckMode,
        findings: Vec<Finding>,
    ) -> bool {
        let verdict = verdict(&findings);
        self.stages.push(StageReport {
            stage,
            verdict,
            mode,
            findings,
        });
        verdict == StageVerdict::Passed
    }

    /// Records a stage whose own computation failed as inconclusive about `root`.
    pub(super) fn record_failure(
        &mut self,
        stage: ValidationStage,
        root: TopologyRef,
        failure: &Failure,
    ) {
        let kind = FindingKind::CheckerFailure {
            message: failure.message().to_owned(),
        };
        let finding = Finding {
            severity: Severity::Inconclusive,
            kind,
            subject: root,
            context: None,
        };
        self.record(stage, CheckMode::Exact, vec![finding]);
    }

    /// The four reports, the unrun ones `NotRun`; `ledger.finish(census, budget)`.
    pub(super) fn finish(
        self,
        census: Vec<EulerCensus>,
        tolerance_budget: Option<f64>,
    ) -> ValidationReport {
        let stages = std::array::from_fn(|k| {
            let recorded = self.stages.get(k).cloned();
            recorded.unwrap_or_else(|| not_run(STAGES[k]))
        });
        ValidationReport {
            stages,
            census,
            tolerance_budget,
        }
    }
}

fn not_run(stage: ValidationStage) -> StageReport {
    let (verdict, mode, findings) = (StageVerdict::NotRun, CheckMode::Exact, Vec::new());
    StageReport {
        stage,
        verdict,
        mode,
        findings,
    }
}

const STAGES: [ValidationStage; 4] = [
    ValidationStage::Topology,
    ValidationStage::Geometry,
    ValidationStage::SelfIntersection,
    ValidationStage::Tolerance,
];

fn verdict(findings: &[Finding]) -> StageVerdict {
    if findings
        .iter()
        .any(|finding| finding.severity == Severity::Defect)
    {
        return StageVerdict::Failed;
    }
    if findings.is_empty() {
        return StageVerdict::Passed;
    }
    StageVerdict::Inconclusive
}
