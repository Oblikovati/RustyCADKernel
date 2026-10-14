// SPDX-License-Identifier: GPL-2.0-only
//! Owned coefficients and caller-qualified identities only; no source topology owners.
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_prism::{
    owned_request::names::*, surface_charts::*, tolerance_inspection::StoredToleranceObservation,
};
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;

pub(crate) struct PrismBodyMember {
    pub(super) key: SemanticMemberKey,
    pub(super) output_slot: Option<SemanticNameSlot>,
    pub(super) rounded_world_point: Option<[f64; 3]>,
    pub(super) curve: Option<CapturedSpaceRepresentation>,
    pub(super) tolerance: Option<StoredToleranceObservation>,
}
pub(crate) struct PrismBodyOccurrence {
    pub(super) key: SemanticUseKey,
    pub(super) orientation: Orientation,
    pub(super) placement: MetricEmbedding<3>,
    pub(super) children: Vec<SemanticUseKey>,
    pub(super) local_vertex: Option<[f64; 3]>,
}
pub(crate) struct PrismBodyFace {
    pub(super) key: SemanticMemberKey,
    pub(super) surface: CapturedPrismSurface,
    pub(super) trims: Vec<PrismBodyTrim>,
}
pub(crate) struct PrismBodyTrim {
    pub(super) occurrence: SemanticUseKey,
    pub(super) role: PrismTrimRole,
    pub(super) pcurve: CapturedMetricCurve<2>,
    pub(super) native_interval: [f64; 2],
    pub(super) space: CapturedSpaceRepresentation,
}
pub(crate) struct PrismBodyEdgeEndpoints {
    pub(super) edge: SemanticMemberKey,
    pub(super) first: SemanticMemberKey,
    pub(super) last: SemanticMemberKey,
}
pub(crate) struct PrismBodyCoedge {
    pub(super) occurrence: SemanticUseKey,
    pub(super) face: SemanticMemberKey,
    pub(super) wire: SemanticMemberKey,
    pub(super) edge: SemanticMemberKey,
    pub(super) reversed: bool,
}
pub(crate) struct PrismBodyWireTraversal {
    pub(super) occurrence: SemanticUseKey,
    pub(super) coedges: Vec<SemanticUseKey>,
}
pub(super) struct PrismBodyRows {
    pub(super) members: Vec<PrismBodyMember>,
    pub(super) occurrences: Vec<PrismBodyOccurrence>,
    pub(super) faces: Vec<PrismBodyFace>,
    pub(super) endpoints: Vec<PrismBodyEdgeEndpoints>,
    pub(super) coedges: Vec<PrismBodyCoedge>,
    pub(super) wires: Vec<PrismBodyWireTraversal>,
}
