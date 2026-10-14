// SPDX-License-Identifier: GPL-2.0-only
use super::super::{AuthoredPrismId, FrozenPrismKey, FrozenUseKey};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;

/// Exact combinatorial evidence only; example: `certificate.invariants()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TopologyInvariant {
    Admission,
    ClosedContainment,
    CyclicWires,
    OppositeEdgeUses,
    CyclicVertexLinks,
    ConnectedFaces,
    SphereEuler,
    Commit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TopologyCheckpoint {
    pub(crate) invariant: TopologyInvariant,
    pub(crate) completed: usize,
}
pub(crate) trait TopologyProgress: Send {
    /// Cancel or fail at an operation-local checkpoint; example: `progress.checkpoint(point)`.
    fn checkpoint(&mut self, point: TopologyCheckpoint) -> Result<bool, Failure>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TopologyNumericalError {
    /// Discrete identities, incidence and integer arithmetic have no metric error budget.
    NotApplicable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TopologyEvidenceKind {
    ExactCombinatorial,
}
#[derive(Debug)]
pub(crate) struct FrozenTopologyCertificate {
    pub(super) source: Box<[u8]>,
    pub(super) operation: AuthoredPrismId,
    pub(super) root: FrozenPrismKey,
    pub(super) counts: [u32; 7],
}
#[derive(Debug)]
pub(crate) enum FrozenTopologyOutcome {
    Certified(Box<FrozenTopologyCertificate>),
    Failed(TopologyFailure),
    Incomplete(TopologyFailure),
    Cancelled {
        operation: AuthoredPrismId,
        checkpoint: TopologyCheckpoint,
    },
}
#[derive(Debug)]
pub(crate) struct TopologyFailure {
    pub(crate) operation: AuthoredPrismId,
    pub(crate) checkpoint: TopologyCheckpoint,
    pub(crate) witness: TopologyWitness,
}
#[derive(Debug)]
pub(crate) struct TopologyWitness {
    pub(crate) entities: Vec<FrozenPrismKey>,
    pub(crate) occurrence: Option<FrozenUseKey>,
    pub(crate) actual: TopologyActual,
    pub(crate) expected: &'static str,
}
#[derive(Debug)]
pub(crate) enum TopologyActual {
    Count(usize),
    Counts([u32; 7]),
    Euler(i64),
    Keys(Vec<FrozenPrismKey>),
    Occurrence(FrozenUseKey),
    Direction(Orientation),
    CaptureInput {
        version: u32,
        import_behavior: rustycad_tkbrep::brep_graph_inc::populate::PCurveImportBehavior,
        ordered: bool,
    },
    Callback(Box<Failure>),
}
pub(super) enum CheckFailure {
    Invalid(TopologyWitness),
    Unsupported(TopologyWitness),
    Cancelled,
}
pub(super) type TopologyResult<T> = Result<T, CheckFailure>;
