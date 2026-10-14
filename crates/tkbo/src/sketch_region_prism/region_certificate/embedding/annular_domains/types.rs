// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularAuthoredTrim {
    pub identity: AnnularTrimIdentity,
    pub operand: PrismAuthoredTrimUse,
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) enum AnnularDomainKind {
    Cap {
        region: u64,
        outer: RegionSourceId,
        inner: RegionSourceId,
        normalized_sweep: f64,
    },
    Side {
        source: RegionSourceId,
        boundary: RegionObservedContext,
        native: [f64; 2],
        stored_sweep: [f64; 2],
        normalized_sweep: [f64; 2],
        height: f64,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularFaceDomain {
    pub identity: AnnularChartIdentity,
    pub kind: AnnularDomainKind,
    pub trims: Vec<AnnularAuthoredTrim>,
}
pub(in crate::sketch_region_prism) struct AnnularDomains {
    pub first: AnnularFaceDomain,
    pub last: AnnularFaceDomain,
    pub outer: AnnularFaceDomain,
    pub inner: AnnularFaceDomain,
}
