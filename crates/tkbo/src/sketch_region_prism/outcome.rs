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
    /// Poll actual execution ownership; `progress.checkpoint(PrismPhase::Copy)` returns stop or typed failure.
    fn checkpoint(&mut self, phase: PrismPhase) -> Result<bool, Failure>;
}
pub(crate) enum PrismCause {
    Mapped {
        phase: PrismPhase,
        failure: Failure,
    },
    Classification(Box<CandidateCause>),
    ValidationRejected(ValidationStage),
    ClassificationProgress {
        failure: Box<Failure>,
        query: Box<CandidateCause>,
    },
    InvalidRequest {
        quantity: &'static str,
        actual: Vec<f64>,
    },
    CyclicBodyContext {
        member: Shape,
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
    MissingImageContext(PrismNativeImage),
    AmbiguousContextPath {
        image: PrismNativeImage,
        contexts: Vec<PrismBodyContext>,
    },
    UnverifiedImageContext {
        image: PrismNativeImage,
        context: PrismBodyContext,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismImageRole {
    Generated,
    First,
    Last,
}
#[derive(Clone)]
pub(crate) struct PrismNativeImage {
    pub query: Shape,
    pub role: PrismImageRole,
    pub returned: ArrangementRelation,
}
#[derive(Clone)]
pub(crate) struct PrismBodyContext {
    pub occurrence: Shape,
    pub ancestors: Vec<Shape>,
    pub child_slots: Vec<usize>,
}
pub(crate) struct PrismImageBinding {
    pub native: PrismNativeImage,
    pub context: PrismBodyContext,
    pub transported: Shape,
}
pub(crate) struct PrismDiagnostic {
    pub body: Shape,
    pub bindings: Vec<PrismImageBinding>,
    pub transport_gaps: Vec<PrismGap>,
    pub relations: Vec<ArrangementRelation>,
    // Native intermediate retirement is not original-member deletion.
    pub native_retired: Vec<ArrangementRelation>,
    pub coverage: Option<OrderedCoverage>,
}
#[derive(Default)]
pub(crate) struct PrismAppliedSettings {
    pub seed: Option<[f64; 2]>,
    pub roots: Option<RootIsolationOptions>,
    pub extrusion: Option<Vector3>,
    pub spatial_limit: Option<f64>,
    pub parameter_limit: Option<f64>,
}
pub(crate) struct PrismLimits {
    pub requested: PrismRequest,
    pub effective: PrismAppliedSettings,
    pub achieved_error: Option<f64>,
}
pub(crate) struct PrismInputHistory {
    pub rejected: Vec<Vec<ArrangementRelation>>,
    pub input_deleted: Vec<OriginalMemberIdentity>,
    pub unrepresented: Vec<ArrangementRelation>,
    pub report: std::sync::Arc<Report>,
    // Metadata aliases are intentional; only the separate completed copy owns isolated geometry.
    pub selected: Option<crate::sketch_region_candidates::RegionCandidate>,
    pub declared_plane: rustycad_tkmath::gp::Plane3,
}
pub(crate) struct PrismCopiedSelection {
    pub base: Shape,
    pub relations: Vec<ArrangementRelation>,
}
pub(crate) struct PrismReceipt {
    pub completion: PrismCompletion,
    pub phase: PrismPhase,
    pub limits: PrismLimits,
    pub classification: Option<CandidateClassification>,
    pub cancelled_query: Option<Box<CandidateCause>>,
    pub copied: Option<PrismCopiedSelection>,
    pub input: PrismInputHistory,
    pub diagnostic: Option<PrismDiagnostic>,
    pub gaps: Vec<PrismGap>,
}
