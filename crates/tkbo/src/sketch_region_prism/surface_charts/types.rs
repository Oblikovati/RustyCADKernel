// SPDX-License-Identifier: GPL-2.0-only
use super::super::PrismImageRole;
use crate::sketch_region::{OriginalMemberIdentity, PartitionOccurrenceIdentity};
use crate::sketch_region_geometry_capture::{CapturedGeometryCause, CapturedMetricCurve};
use crate::sketch_region_source_conformance::{CapturedPlaneSupport, CapturedSpaceRepresentation};
use rustycad_tkmath::predicates::roots::metric_bounds::{
    CurveIdentityDecision, MetricEmbedding, ResidualError,
};
// Snapshot keys use immutable Location values; topology addresses remain operation-local.
pub(crate) type PrismChartSources = Result<CapturedPrismSources, std::sync::Arc<PrismSurfaceCause>>;
pub(crate) struct CapturedPrismSources {
    pub face: PartitionOccurrenceIdentity,
    pub support: CapturedPlaneSupport,
    pub boundaries: Vec<CapturedSourceBoundary>,
}
pub(crate) struct CapturedSourceBoundary {
    pub occurrence: PartitionOccurrenceIdentity,
    pub space: CapturedSpaceRepresentation,
    pub pcurve: CapturedMetricCurve<2>,
    pub interval: [f64; 2],
    pub endpoints: Vec<(PartitionOccurrenceIdentity, f64)>,
}

// These scalar inspections do not own or publish the mutable Shape aliases in PrismReceipt.
// Scalar captures/proofs have no geometry handles and do not authorize later mutated shapes.
pub(crate) struct PrismSurfaceInspection {
    pub version: u32,
    pub entries: Vec<PrismSurfaceEntry>,
    pub uncovered_faces: Vec<(PartitionOccurrenceIdentity, PrismSurfaceCause)>,
    pub face_count: usize,
    pub scanned_all_faces: bool,
}
pub(crate) struct PrismSurfaceEntry {
    pub source: PartitionOccurrenceIdentity,
    pub output: PartitionOccurrenceIdentity,
    pub role: PrismImageRole,
    pub parents: Vec<OriginalMemberIdentity>,
    pub captured: Option<CapturedSurfaceChart>,
    pub proof: PrismSurfaceProof,
    pub decision: Result<(), PrismSurfaceCause>,
}
pub(crate) struct CapturedSurfaceChart {
    pub source: CapturedChartSource,
    pub actual: CapturedPrismSurface,
    pub trims: Vec<CapturedPrismTrim>,
    pub extrusion: [f64; 3],
}
pub(crate) enum CapturedChartSource {
    Cap(CapturedPlaneSupport),
    Side(CapturedSpaceRepresentation),
}
pub(crate) enum CapturedPrismSurface {
    Plane(CapturedPlaneSupport),
    Extrusion {
        basis: CapturedMetricCurve<3>,
        direction: [f64; 3],
        placement: MetricEmbedding<3>,
    },
}
pub(crate) struct CapturedPrismTrim {
    pub output: PartitionOccurrenceIdentity,
    pub source: PartitionOccurrenceIdentity,
    pub parents: Vec<OriginalMemberIdentity>,
    pub role: PrismTrimRole,
    pub pcurve: CapturedMetricCurve<2>,
    pub interval: [f64; 2],
    pub space: CapturedSpaceRepresentation,
    pub expected: CapturedTrimExpectation,
}
pub(crate) enum CapturedTrimExpectation {
    Cap {
        pcurve: CapturedMetricCurve<2>,
        interval: [f64; 2],
        space: Box<CapturedSpaceRepresentation>,
    },
    SideGenerating,
    SideDirecting {
        parameter: f64,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismTrimRole {
    First,
    Last,
    Start,
    End,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismChartStage {
    CapSupportU,
    CapSupportV,
    SideBasis,
    SideDirection,
    TrimPCurve,
    TrimSpaceCurve,
}
#[derive(Debug, Default)]
pub(crate) struct PrismSurfaceProof {
    pub identities: Vec<PrismChartStage>,
    pub trim_roles: Vec<PrismTrimRole>,
    pub side_height: Option<f64>,
}
#[derive(Debug)]
pub(crate) enum PrismSurfaceCause {
    SourceSnapshot(std::sync::Arc<PrismSurfaceCause>),
    Capture(CapturedGeometryCause),
    Unsupported {
        representation: &'static str,
        actual: String,
    },
    AlternateEvaluator {
        representation: &'static str,
    },
    MissingHistory {
        entity: &'static str,
    },
    AmbiguousHistory {
        entity: &'static str,
        count: usize,
    },
    Count {
        entity: &'static str,
        actual: usize,
        expected: usize,
    },
    Range {
        actual: [f64; 2],
        expected: [f64; 2],
    },
    InvalidHeight {
        actual: f64,
        expected: &'static str,
    },
    SameParameterRequired,
    Identity {
        stage: PrismChartStage,
        decision: CurveIdentityDecision,
    },
    Exact(ResidualError),
}
impl From<CapturedGeometryCause> for PrismSurfaceCause {
    fn from(cause: CapturedGeometryCause) -> Self {
        // Keep the original capture cause for the actual source/output representation.
        Self::Capture(cause)
    }
}
impl From<rustycad_tkernel::standard::Failure> for PrismSurfaceCause {
    fn from(cause: rustycad_tkernel::standard::Failure) -> Self {
        // Mapped topology failures remain distinguishable from coefficient mismatches.
        Self::Capture(cause.into())
    }
}
impl From<ResidualError> for PrismSurfaceCause {
    fn from(cause: ResidualError) -> Self {
        // Cancellation and invalid coefficients retain their numerical failure class.
        Self::Exact(cause)
    }
}
