// SPDX-License-Identifier: GPL-2.0-only
use super::super::surface_charts::{
    CapturedChartSource, CapturedPrismSurface, CapturedTrimExpectation, PrismSurfaceCause,
    PrismSurfaceProof, PrismTrimRole,
};
use super::super::{PrismBodyContext, PrismImageRole};
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use crate::sketch_region_validation::{StageCoverage, ValidationStage};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;

/// Explicit caller names; zero is deliberately invalid. Example: `AuthoredPrismId(703)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AuthoredPrismId(pub u64);
/// A definition name, derived only from authored identity and observed sweep role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FrozenPrismKey {
    pub operation: AuthoredPrismId,
    pub source: AuthoredPrismId,
    pub role: FrozenSweepRole,
    pub kind: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum FrozenSweepRole {
    First,
    Last,
    Generated,
}
/// Full directed ancestry is identity; sibling position is only stored ordering.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FrozenUseKey(pub Vec<(FrozenPrismKey, AuthoredPrismId)>);
/// Lookup/provenance only: the actual addresses never enter frozen identity.
pub(crate) struct PrismOriginalBinding {
    pub actual: OriginalMemberIdentity,
    pub member: AuthoredPrismId,
    pub directed_use: AuthoredPrismId,
}
pub(crate) struct PrismSourceBinding {
    pub actual: PartitionOccurrenceIdentity,
    pub member: AuthoredPrismId,
}
pub(crate) struct PrismDirectedBinding {
    pub actual: PrismBodyContext,
    pub authored_use: AuthoredPrismId,
}
/// These IDs must be authored before lookup, never numbered from returned traversal.
/// Example: bind a named sketch edge and its authored start/end uses, then observed sweep roles.
pub(crate) struct PrismPublicationBindings {
    pub operation: AuthoredPrismId,
    pub selected_region: AuthoredPrismId,
    pub originals: Vec<PrismOriginalBinding>,
    pub sources: Vec<PrismSourceBinding>,
    pub directed: Vec<PrismDirectedBinding>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FreezePhase {
    Admission,
    Identity,
    Geometry,
    Incidence,
    Evidence,
    Commit,
}
pub(crate) trait FreezeProgress: Send {
    /// Poll one bounded phase; example: `progress.checkpoint(FreezePhase::Geometry)`.
    fn checkpoint(&mut self, phase: FreezePhase) -> Result<bool, Failure>;
}
pub(crate) enum PrismFreezeOutcome {
    Captured(FrozenPrismProjection),
    Incomplete(PrismFreezeFailure),
    Failed(PrismFreezeFailure),
    Cancelled {
        operation: AuthoredPrismId,
        phase: FreezePhase,
    },
}
#[derive(Debug)]
pub(crate) struct PrismFreezeFailure {
    pub operation: AuthoredPrismId,
    pub phase: FreezePhase,
    pub entity: Option<FrozenPrismKey>,
    pub cause: PrismFreezeCause,
}
#[derive(Debug)]
pub(crate) enum PrismFreezeCause {
    Missing {
        required: &'static str,
        actual: usize,
    },
    Identity {
        required: &'static str,
        actual: usize,
    },
    Unsupported {
        representation: &'static str,
        actual: &'static str,
    },
    InvalidScalar {
        quantity: &'static str,
        actual: f64,
    },
    UnsupportedOrientation {
        actual: Orientation,
        expected: &'static str,
    },
    UnsupportedDiagnostic {
        stage: Option<ValidationStage>,
        actual: &'static str,
    },
    StaleCapture {
        representation: &'static str,
    },
    GraphStatus(rustycad_tkbrep::brep_graph::shapes_view::AddStatus),
    GraphAudit(Vec<super::graph::FrozenGraphIssue>),
    Count {
        entity: &'static str,
        actual: usize,
        expected: usize,
    },
    Endpoint(rustycad_tkmath::predicates::roots::metric_bounds::ResidualError),
    EndpointMismatch {
        parameter: f64,
        point: [f64; 3],
        decision: rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision,
    },
    Native(Failure),
    Surface(PrismSurfaceCause),
}
#[derive(Clone)]
pub(crate) struct FrozenPrismProjection {
    pub(super) version: u32,
    pub(super) operation: AuthoredPrismId,
    pub(super) region: AuthoredPrismId,
    pub(super) nodes: Vec<FrozenPrismNode>,
    pub(super) uses: Vec<FrozenPrismUse>,
    pub(super) graph: super::graph::FrozenGraphIncidence,
    pub(super) faces: Vec<FrozenPrismFace>,
    pub(super) history: FrozenPrismHistory,
    pub(super) coverage: super::coverage::FrozenPrismCoverage,
    pub(super) limits: FrozenPrismLimits,
    pub(super) endpoints: Vec<super::endpoints::FrozenEndpointProof>,
}
#[derive(Clone)]
pub(crate) struct FrozenPrismNode {
    pub(crate) key: FrozenPrismKey,
    pub(crate) point: Option<[f64; 3]>,
    pub(crate) tolerance: Option<f64>,
    pub(crate) curve: Option<CapturedSpaceRepresentation>,
}
#[derive(Clone)]
pub(crate) struct FrozenPrismUse {
    pub(crate) key: FrozenUseKey,
    pub(crate) orientation: Orientation,
    pub(crate) placement: MetricEmbedding<3>,
    pub(crate) children: Vec<FrozenUseKey>,
}
#[derive(Clone)]
pub(crate) struct FrozenPrismFace {
    pub(crate) key: FrozenPrismKey,
    /// Source operands are named by key.source and retained for replay of the exact proof.
    pub(crate) source: CapturedChartSource,
    pub(crate) surface: CapturedPrismSurface,
    pub(crate) trims: Vec<FrozenPrismTrim>,
    pub(crate) proof: PrismSurfaceProof,
}
#[derive(Clone)]
pub(crate) struct FrozenPrismTrim {
    pub(crate) occurrence: FrozenUseKey,
    pub(crate) source: AuthoredPrismId,
    pub(crate) parents: Vec<FrozenOriginalKey>,
    pub(crate) role: PrismTrimRole,
    pub(crate) expected: CapturedTrimExpectation,
    pub(crate) pcurve: CapturedMetricCurve<2>,
    pub(crate) interval: [f64; 2],
    pub(crate) space: CapturedSpaceRepresentation,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FrozenOriginalKey {
    pub source: u64,
    pub member: AuthoredPrismId,
    pub directed_use: AuthoredPrismId,
}
#[derive(Clone, Debug)]
pub(crate) struct FrozenPrismHistory {
    pub(crate) originals: Vec<FrozenOriginalOccurrence>,
    pub(crate) rejected_unbound: Vec<Vec<FrozenRejectedRelation>>,
    pub(crate) derived: Vec<FrozenDerivedRelation>,
    pub(crate) deleted: Vec<FrozenOriginalKey>,
}
#[derive(Clone)]
pub(crate) struct FrozenOriginalOccurrence {
    pub(crate) key: FrozenOriginalKey,
    pub(crate) orientation: Orientation,
    pub(crate) placement: MetricEmbedding<3>,
}
/// Rejected selection provenance has no invented output identity or retained geometry owner.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FrozenRejectedRelation {
    pub(crate) output_kind: u8,
    pub(crate) parents: [Vec<FrozenOriginalKey>; 3],
}
/// All original parents are retained; the mapped class is provenance of derivation.
#[derive(Clone, Debug)]
pub(crate) struct FrozenDerivedRelation {
    pub(crate) output: FrozenUseKey,
    pub(crate) preserved: Vec<FrozenOriginalKey>,
    pub(crate) modified: Vec<FrozenOriginalKey>,
    pub(crate) generated: Vec<FrozenOriginalKey>,
}
#[derive(Clone, Copy)]
pub(crate) struct FrozenPrismLimits {
    pub(crate) seed: [f64; 2],
    pub(crate) extrusion: [f64; 3],
    pub(crate) roots: RootIsolationOptions,
    pub(crate) spatial_limit: f64,
    pub(crate) parameter_limit: f64,
    pub(crate) effective_seed: Option<[f64; 2]>,
    pub(crate) effective_extrusion: Option<[f64; 3]>,
    pub(crate) effective_roots: Option<RootIsolationOptions>,
    pub(crate) effective_spatial: Option<f64>,
    pub(crate) effective_parameter: Option<f64>,
    pub(crate) achieved_error: Option<f64>,
}
impl From<PrismImageRole> for FrozenSweepRole {
    fn from(role: PrismImageRole) -> Self {
        match role {
            PrismImageRole::First => Self::First,
            PrismImageRole::Last => Self::Last,
            PrismImageRole::Generated => Self::Generated,
        }
    }
}
