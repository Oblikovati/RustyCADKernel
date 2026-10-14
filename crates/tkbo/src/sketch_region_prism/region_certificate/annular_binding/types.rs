// SPDX-License-Identifier: GPL-2.0-only
//! Full directed keys are operation-local correspondence, never address-derived portable names.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularChartIdentity {
    pub source: RegionObservedContext,
    pub output: RegionObservedContext,
    pub role: PrismImageRole,
}
pub(in crate::sketch_region_prism) struct AnnularCopiedMember {
    pub key: TwoLoopMemberKey,
    pub selected: RegionObservedContext,
    pub copied: RegionObservedContext,
}
pub(super) struct AnnularCopyCapture {
    pub members: Vec<AnnularCopiedMember>,
    pub selected: super::super::super::surface_charts::CapturedPrismSources,
    pub copied: super::super::super::surface_charts::CapturedPrismSources,
}
pub(in crate::sketch_region_prism) struct AnnularPrismBinding {
    pub(super) region: Box<TwoLoopRegionCertificate>,
    pub(super) operands: AnnularPrismOperands,
    pub(super) replay: Vec<u8>,
}
pub(super) struct AnnularPrismOperands {
    pub copy: AnnularCopyCapture,
    pub boundaries: Vec<AnnularBoundaryBinding>,
    pub charts: Vec<AnnularBoundChart>,
    pub body_contexts: Vec<RegionObservedContext>,
    pub limits: super::super::super::frozen::FrozenPrismLimits,
    pub signed_jacobian: rustycad_tkmath::predicates::roots::DyadicBound,
    pub parents: AnnularParentCapture,
}
pub(in crate::sketch_region_prism) struct AnnularParentCapture {
    pub copies: Vec<AnnularCopyParents>,
    pub images: Vec<AnnularImageParents>,
    pub retained_units: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularCopyParents {
    pub key: TwoLoopMemberKey,
    pub selected: [Vec<TwoLoopOriginalUse>; 3],
    pub copied: [Vec<TwoLoopOriginalUse>; 3],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularImageParents {
    pub key: TwoLoopMemberKey,
    pub identity: AnnularChartIdentity,
    pub parents: [Vec<TwoLoopOriginalUse>; 3],
}
pub(in crate::sketch_region_prism) struct AnnularBoundaryBinding {
    pub source: RegionSourceId,
    pub wire: u64,
    pub selected: RegionObservedContext,
    pub copied: RegionObservedContext,
    pub copied_wire: RegionObservedContext,
    pub endpoints: Vec<AnnularNativeEndpoint>,
}
pub(in crate::sketch_region_prism) struct AnnularNativeEndpoint {
    pub context: RegionObservedContext,
    pub parameter: f64,
    pub local_point: [f64; 3],
    pub placement: rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding<3>,
}
pub(in crate::sketch_region_prism) struct AnnularBoundChart {
    pub identity: AnnularChartIdentity,
    pub source_boundary: Option<RegionSourceId>,
    pub chart: super::super::current_binding::BoundPrismChart,
    pub trims: Vec<AnnularTrimIdentity>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularTrimIdentity {
    pub source: RegionObservedContext,
    pub output: RegionObservedContext,
    pub role: super::super::super::surface_charts::PrismTrimRole,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularBindingStage {
    Admission,
    Copy,
    Charts,
    Commit,
}
pub(in crate::sketch_region_prism) trait AnnularBindingProgress:
    Send
{
    /// Observe bounded operation work; `progress.checkpoint(AnnularBindingStage::Copy)`.
    fn checkpoint(
        &mut self,
        stage: AnnularBindingStage,
    ) -> Result<bool, rustycad_tkernel::standard::Failure>;
}
