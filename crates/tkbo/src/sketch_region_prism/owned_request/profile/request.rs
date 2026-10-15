// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0066 part C owned profile inputs: edges, authored loops of directed uses and regions.
use super::*;

/// One owned planar Bézier edge; polynomial or positive-weight rational.
/// Example: `OwnedProfileEdge { source: CallerBoundaryId(11), start: CallerVertexId(1), .. }`.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct OwnedProfileEdge {
    pub source: CallerBoundaryId,
    pub revision: CallerSourceRevision,
    pub world_poles: Vec<[f64; 3]>,
    /// One positive finite weight per pole, or `None` for a polynomial edge.
    pub world_weights: Option<Vec<f64>>,
    /// Caller vertex at native parameter 0; equal ids share one constructed vertex.
    pub start: CallerVertexId,
    /// Caller vertex at native parameter 1.
    pub end: CallerVertexId,
}
/// One directed use of an edge inside an authored loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct OwnedProfileUse {
    pub edge: CallerBoundaryId,
    pub reversed: bool,
}
/// Authored closed loop: consecutive uses share a caller vertex, cyclically.
/// The region proof certifies it; authoring alone proves nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct OwnedProfileLoop {
    pub name: u64,
    pub uses: Vec<OwnedProfileUse>,
}
/// One selected material region, named by the caller and located by a seed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct OwnedProfileRegion {
    pub name: u64,
    pub seed: [f64; 2],
}
/// Explicit child policies; no implicit default is substituted.
#[derive(Clone, Copy)]
pub(in crate::sketch_region_prism) struct OwnedProfilePolicy {
    pub census: CensusPolicy,
    pub source: SourceConformancePolicy,
    pub contacts: ContactTransportPolicy,
    pub region: LoopRegionOptions,
}
/// Bounded version-1 profile request.
#[derive(Clone)]
pub(in crate::sketch_region_prism) struct OwnedProfileRequest {
    pub version: u32,
    pub operation: CallerOperationId,
    pub edges: Vec<OwnedProfileEdge>,
    pub loops: Vec<OwnedProfileLoop>,
    pub regions: Vec<OwnedProfileRegion>,
    pub declared_plane: Plane3,
    pub policy: OwnedProfilePolicy,
}
impl OwnedProfileRequest {
    /// The edge authored under `id`, if any.
    pub(in crate::sketch_region_prism) fn edge(
        &self,
        id: CallerBoundaryId,
    ) -> Option<&OwnedProfileEdge> {
        self.edges.iter().find(|edge| edge.source == id)
    }
}
