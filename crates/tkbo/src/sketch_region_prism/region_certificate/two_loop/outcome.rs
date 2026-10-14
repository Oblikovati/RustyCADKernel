// SPDX-License-Identifier: GPL-2.0-only
//! Owned exact evidence remains distinct from operation-local legacy refusal diagnostics.
use super::{nesting::TwoLoopNestingProof, pair_types::*, request::*};
use crate::sketch_region::RegionSourceId;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::{CapturedPlaneSupport, CapturedSpaceRepresentation};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

pub(crate) enum TwoLoopRegionOutcome<'a> {
    Inspected(Box<TwoLoopRegionCertificate>),
    Incomplete(Box<super::failure::TwoLoopRegionFailure<'a>>),
    Failed(Box<super::failure::TwoLoopRegionFailure<'a>>),
    Cancelled(Box<super::failure::TwoLoopRegionFailure<'a>>),
}
pub(crate) struct TwoLoopRegionCertificate {
    pub(super) operation: u64,
    pub(super) requested: TwoLoopRegionOptions,
    pub(super) sources: Vec<TwoLoopSourceProof>,
    pub(super) domains: Vec<TwoLoopPairProof>,
    pub(super) candidates: Vec<TwoLoopCandidateProof>,
    pub(super) nesting: TwoLoopNestingProof,
    pub(super) retained_units: usize,
    pub(super) replay: Vec<u8>,
    pub(super) binding: Vec<u8>,
    pub(super) selected: u64,
    pub(super) census: super::census_settings::TwoLoopCensusSettings,
}
pub(crate) struct TwoLoopSourceProof {
    pub(crate) name: RegionSourceId,
    pub(crate) original: CapturedSpaceRepresentation,
    pub(crate) copied: CapturedSpaceRepresentation,
    pub(crate) pcurve: CapturedMetricCurve<2>,
    pub(crate) support: CapturedPlaneSupport,
    pub(crate) orientation: Orientation,
    pub(crate) closure: Vec<TwoLoopEndpointProof>,
    pub(crate) copied_closure: Vec<TwoLoopEndpointProof>,
    pub(crate) source_policy: crate::sketch_region_source_conformance::SourceConformancePolicy,
    pub(crate) stages: Vec<TwoLoopSourceStage>,
    pub(crate) raw_tolerances: [f64; 2],
}
#[derive(Debug)]
pub(crate) struct TwoLoopSourceStage {
    pub(crate) stage: crate::sketch_region_source_conformance::ConstructionStage,
    pub(crate) requested_limit: f64,
    pub(crate) effective_limit: f64,
    pub(crate) effective_roots: rustycad_tkmath::predicates::roots::RootIsolationOptions,
    pub(crate) bisections: u64,
    pub(crate) residual: ResidualDecision,
}
pub(crate) struct TwoLoopEndpointProof {
    pub(crate) parameter: f64,
    pub(crate) orientation: Orientation,
    pub(crate) local_point: [f64; 3],
    pub(crate) placement: rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding<3>,
    pub(crate) residual: ResidualDecision,
    pub(crate) bisections: u64,
    pub(crate) raw_tolerance: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TwoLoopCandidateDisposition {
    SelectedAnnulus,
    RejectedInnerDomain,
    RejectedOuterComplement,
}
pub(crate) struct TwoLoopCandidateProof {
    pub(crate) name: u64,
    pub(crate) orientation: Orientation,
    pub(crate) support: CapturedPlaneSupport,
    pub(crate) wires: Vec<TwoLoopWireProof>,
    pub(crate) disposition: TwoLoopCandidateDisposition,
    /// Outer/inner winding contributions normalized by each proved native Jordan orientation.
    pub(crate) signed_signature: [i64; 2],
    /// At the certified material seed: outer interior contributes one, inner exterior zero.
    pub(crate) material_winding: i64,
    pub(crate) members: Vec<super::member_types::TwoLoopMemberProof>,
}
pub(crate) struct TwoLoopWireProof {
    pub(crate) name: u64,
    pub(crate) source: RegionSourceId,
    pub(crate) wire_orientation: Orientation,
    pub(crate) edge_orientation: Orientation,
    pub(crate) space: CapturedSpaceRepresentation,
    pub(crate) pcurve: CapturedMetricCurve<2>,
    pub(crate) closure: Vec<TwoLoopEndpointProof>,
    pub(crate) parents: Vec<RegionSourceId>,
}
