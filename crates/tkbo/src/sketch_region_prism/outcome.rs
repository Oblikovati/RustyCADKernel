// SPDX-License-Identifier: GPL-2.0-only
use crate::sketch_region::{ArrangementRelation, OriginalMemberIdentity};
use crate::sketch_region_candidates::{
    CandidateCause, CandidateClassification, MissingCertificate, SeedSelection,
};
use crate::sketch_region_validation::{OrderedCoverage, ValidationStage};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{message::Report, standard::Failure};
use rustycad_tkmath::{gp::Vector3, predicates::roots::RootIsolationOptions};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismPhase {
    Admission,
    Selection,
    Copy,
    Sweep,
    History,
    Validation,
}
#[derive(Clone, Copy)]
pub(crate) struct PrismRequest {
    pub seed: [f64; 2],
    pub extrusion: Vector3,
    pub roots: RootIsolationOptions,
    pub spatial_limit: f64,
    pub parameter_limit: f64,
}
pub(crate) trait PrismProgress {
    fn checkpoint(&mut self, phase: PrismPhase) -> Result<bool, Failure>;
}
pub(crate) enum PrismCause {
    Mapped {
        phase: PrismPhase,
        failure: Failure,
    },
    Classification(CandidateCause),
    ValidationRejected(ValidationStage),
    ClassificationProgress {
        failure: Failure,
        query: CandidateCause,
    },
    InvalidRequest {
        quantity: &'static str,
        actual: Vec<f64>,
    },
    MissingCopy {
        member: Shape,
        images: usize,
    },
}
pub(crate) enum PrismCompletion {
    Incomplete,
    NoBody(SeedSelection),
    Failed(PrismCause),
    Cancelled(PrismPhase),
}
pub(crate) enum PrismGap {
    Candidate(MissingCertificate),
    UnattributedSweepOccurrence(Shape),
    UnrepresentedInput,
    PrismInjectivityAndSurfaceCharts,
    AchievedErrorUnknown,
    ComposedDeletionCoverage,
}
pub(crate) struct PrismDiagnostic {
    pub body: Shape,
    pub relations: Vec<ArrangementRelation>,
    // Native intermediate retirement is not original-member deletion.
    pub native_retired: Vec<ArrangementRelation>,
    pub coverage: Option<OrderedCoverage>,
}
pub(crate) struct PrismLimits {
    pub requested: PrismRequest,
    pub effective: PrismRequest,
    pub achieved_error: Option<f64>,
}
pub(crate) struct PrismInputHistory {
    pub rejected: Vec<Vec<ArrangementRelation>>,
    pub input_deleted: Vec<OriginalMemberIdentity>,
    pub unrepresented: Vec<ArrangementRelation>,
    pub report: std::sync::Arc<Report>,
}
pub(crate) struct PrismReceipt {
    pub completion: PrismCompletion,
    pub phase: PrismPhase,
    pub limits: PrismLimits,
    pub classification: Option<CandidateClassification>,
    pub input: PrismInputHistory,
    pub diagnostic: Option<PrismDiagnostic>,
    pub gaps: Vec<PrismGap>,
}
