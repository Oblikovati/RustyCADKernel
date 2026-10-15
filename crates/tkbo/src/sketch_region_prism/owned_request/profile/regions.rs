// SPDX-License-Identifier: GPL-2.0-only
//! Run the general loop region proof on a prepared profile's own transaction.
use super::*;
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
