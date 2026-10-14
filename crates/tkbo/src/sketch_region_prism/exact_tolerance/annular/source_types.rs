// SPDX-License-Identifier: GPL-2.0-only
//! Native source roles and authored candidate/wire keys never masquerade as V1 contact IDs.
use crate::sketch_region::RegionSourceId;
use crate::sketch_region_prism::exact_tolerance::{CanonicalEndpointSide, frames::*, scalar::*};
use crate::sketch_region_source_conformance::ConstructionStage;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::ResidualDecision};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularClosureStage {
    Original,
    Census,
    Candidate { candidate: u64, wire: u64 },
    PrismCopy { candidate: u64, wire: u64 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularSourceContext {
    pub source: RegionSourceId,
    pub stage: AnnularClosureStage,
    pub native: Option<CanonicalEndpointSide>,
}
pub(in crate::sketch_region_prism) struct AnnularSupportEvidence {
    pub context: AnnularSourceContext,
    pub frame: UsableToleranceFrame,
}
pub(in crate::sketch_region_prism) struct AnnularClosureEvidence {
    pub context: AnnularSourceContext,
    pub reused_from: Option<AnnularSourceContext>,
    pub parameter: f64,
    pub orientation: Orientation,
    pub local_point: [f64; 3],
    /// Metadata from the reused source proof; copied actual metadata remains in the sealed join.
    pub raw_tolerance: f64,
    pub certificate: ExactEndpointCertificate,
}
pub(in crate::sketch_region_prism) struct AnnularSourceReport {
    pub source: RegionSourceId,
    pub stage: ConstructionStage,
    pub requested_limit: f64,
    pub effective_limit: f64,
    pub effective_roots: RootIsolationOptions,
    pub bisections: u64,
    pub residual: ResidualDecision,
}
