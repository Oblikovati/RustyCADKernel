// SPDX-License-Identifier: GPL-2.0-only
//! Retained scalar copies bind numerical proofs; shape handles are identity aliases, not isolation evidence.
use crate::sketch_region::ArrangementRelation;
use crate::sketch_region_candidates::CandidateSegment;
use rustycad_tkmath::predicates::roots::bezier_pair::*;

pub(crate) struct PairedCensusEvidence {
    pub requested: BezierPairPolicy,
    pub occurrences: Vec<PairedOccurrenceSnapshot>,
    pub reports: Vec<PairedStoredReport>,
    pub reused_reports: u64,
}
pub(crate) struct PairedOccurrenceSnapshot {
    pub source: CandidateSegment,
    pub relations: Vec<ArrangementRelation>,
    pub physical_source: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PairedQueryMode {
    Distinct,
    SelfOffDiagonal,
}
pub(crate) struct PairedStoredReport {
    pub sources: [usize; 2],
    pub mode: PairedQueryMode,
    pub effective: BezierPairPolicy,
    pub outcome: Result<BezierPairReport, Box<BezierPairCaptureFailure>>,
}
impl PairedStoredReport {
    /// Capture validation performs no isolation; `record.completed_work()` charges only actual solver work.
    pub(crate) fn completed_work(&self) -> u64 {
        self.outcome
            .as_ref()
            .map_or(0, |report| u64::from(report.work().completed))
    }
    /// Preserve malformed capture versus solver noncompletion; `record.completion()` never equates either with empty.
    pub(crate) fn completion(&self) -> BezierPairCompletion {
        self.outcome
            .as_ref()
            .map_or(BezierPairCompletion::Invalid, |report| report.completion())
    }
}
