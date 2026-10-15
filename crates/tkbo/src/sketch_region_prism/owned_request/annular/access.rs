// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed views over prepared owners; the region request borrows only this preparation.
use super::*;

impl PreparedAnnularPrism {
    /// Borrow the authored role-bound region identities; `prepared.identities().selected`.
    pub(in crate::sketch_region_prism) fn identities(&self) -> &TwoLoopRegionIdentities {
        &self.identities
    }
    /// Borrow the retained sweep receipt; `prepared.receipt().diagnostic`.
    pub(in crate::sketch_region_prism) fn receipt(&self) -> &PrismReceipt {
        &self.receipt
    }
    /// Borrow the original owned request; `prepared.request().regions.annulus`.
    pub(in crate::sketch_region_prism) fn request(&self) -> &OwnedAnnularPrismRequest {
        &self.retained.request
    }
    /// Run against the two-loop request over these owners, outer then inner by authored role.
    /// Example: `prepared.with_region(|region, receipt| inspect_two_loop_region(region, p))`.
    pub(in crate::sketch_region_prism) fn with_region<T>(
        &self,
        run: impl FnOnce(&TwoLoopRegionRequest<'_, '_>, &PrismReceipt) -> T,
    ) -> T {
        let sources = self.retained.arrangement_sources();
        let request = &self.retained.request;
        let region = TwoLoopRegionRequest {
            bridge: &self.bridge,
            sources: &sources,
            identities: &self.identities,
            inner_interior: request.inner_interior,
            material_seed: request.material_seed,
            requested: request.policy.region,
        };
        run(&region, &self.receipt)
    }
}
impl AnnularPreparationFailure {
    /// Borrow the retained sweep receipt when the prism stage was reached; `failure.receipt()`.
    pub(in crate::sketch_region_prism) fn receipt(&self) -> Option<&PrismReceipt> {
        self.retained.prism.as_ref()
    }
    /// Borrow the original owned request; `failure.request().regions.annulus`.
    pub(in crate::sketch_region_prism) fn request(&self) -> &OwnedAnnularPrismRequest {
        &self.retained.request
    }
    /// List retained partial-transaction owners in pipeline order; `failure.retained_stages()`.
    /// An empty list does not prove a nested cause was projected.
    pub(in crate::sketch_region_prism) fn retained_stages(&self) -> Vec<RetainedPreparationStage> {
        use RetainedPreparationStage::*;
        let retained = &self.retained;
        [
            (ConstructedSource, !retained.sources.is_empty()),
            (Census, retained.census.is_some()),
            (Partition, retained.partition.is_some()),
            (ContactTransport, retained.bridge.is_some()),
            (RegionIdentities, retained.identities.is_some()),
            (Prism, retained.prism.is_some()),
        ]
        .into_iter()
        .filter_map(|(stage, present)| present.then_some(stage))
        .collect()
    }
}
