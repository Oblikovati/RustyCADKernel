// SPDX-License-Identifier: GPL-2.0-only
//! Run the general loop region proof on a prepared profile's own transaction.
use super::*;
use crate::sketch_region_prism::region_certificate::loop_region::graph::{
    GraphRegionCertificate, GraphRegionRequest, inspect_plane_graph,
};
use crate::sketch_region_prism::region_certificate::loop_region::{
    AuthoredLoop, AuthoredLoopUse, AuthoredRegion, LoopRegionCertificate, LoopRegionCheck,
    LoopRegionRequest, inspect_loop_regions,
};

impl PreparedProfile {
    /// Certify the authored loops and selected regions; `prepared.inspect_regions(|| false)`.
    pub(in crate::sketch_region_prism) fn inspect_regions(
        &self,
        cancelled: impl FnMut() -> bool,
    ) -> LoopRegionCheck<LoopRegionCertificate> {
        let loops: Vec<AuthoredLoop> = self.request.loops.iter().map(authored_loop).collect();
        let regions: Vec<AuthoredRegion> = self
            .request
            .regions
            .iter()
            .map(|region| AuthoredRegion {
                name: region.name,
                seed: region.seed,
            })
            .collect();
        let sources = self.sources();
        let request = LoopRegionRequest {
            bridge: &self.bridge,
            sources: &sources,
            loops: &loops,
            regions: &regions,
            options: self.request.policy.region,
        };
        inspect_loop_regions(&request, cancelled)
    }
}
impl PreparedProfile {
    /// Certify every bounded face of a version-2 plane graph (ADR 0084);
    /// `prepared.inspect_plane_graph(|| false)`.
    pub(in crate::sketch_region_prism) fn inspect_plane_graph(
        &self,
        cancelled: impl FnMut() -> bool,
    ) -> LoopRegionCheck<GraphRegionCertificate> {
        let ends: Vec<[u64; 2]> = self
            .request
            .edges
            .iter()
            .map(|edge| [edge.start.0, edge.end.0])
            .collect();
        let sources = self.sources();
        let request = GraphRegionRequest {
            bridge: &self.bridge,
            sources: &sources,
            ends: &ends,
            options: self.request.policy.region,
        };
        inspect_plane_graph(&request, cancelled)
    }
}
fn authored_loop(owned: &OwnedProfileLoop) -> AuthoredLoop {
    let uses = owned.uses.iter().map(|directed| AuthoredLoopUse {
        source: RegionSourceId(directed.edge.0),
        reversed: directed.reversed,
    });
    AuthoredLoop {
        name: owned.name,
        uses: uses.collect(),
    }
}
