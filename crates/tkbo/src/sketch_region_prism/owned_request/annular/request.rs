// SPDX-License-Identifier: GPL-2.0-only
//! Source-owned annular inputs: explicit outer/inner boundary roles and authored region names.
use super::*;

/// Authored names of the three semantic regions and their four directed boundary wires.
/// Example: `AnnularRegionNames { annulus: 301, inner_domain: 302, .. }`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularRegionNames {
    pub annulus: u64,
    pub inner_domain: u64,
    pub outer_complement: u64,
    pub annulus_outer_wire: u64,
    pub annulus_inner_wire: u64,
    pub inner_domain_wire: u64,
    pub outer_complement_wire: u64,
}
impl AnnularRegionNames {
    /// Every authored name, in role order; `names.all()` is checked for zero and duplicates.
    pub(super) fn all(&self) -> [u64; 7] {
        [
            self.annulus,
            self.inner_domain,
            self.outer_complement,
            self.annulus_outer_wire,
            self.annulus_inner_wire,
            self.inner_domain_wire,
            self.outer_complement_wire,
        ]
    }
}
/// Explicit child policies; no implicit default is substituted.
#[derive(Clone, Copy)]
pub(in crate::sketch_region_prism) struct OwnedAnnularPolicy {
    pub census: CensusPolicy,
    pub source: SourceConformancePolicy,
    pub contacts: ContactTransportPolicy,
    pub classification_roots: RootIsolationOptions,
    pub spatial_limit: f64,
    pub parameter_limit: f64,
    pub region: TwoLoopRegionOptions,
}
/// Bounded V1 annular request; the outer and inner roles are explicit, never request order.
#[derive(Clone)]
pub(in crate::sketch_region_prism) struct OwnedAnnularPrismRequest {
    pub version: u32,
    pub operation: CallerOperationId,
    pub outer: OwnedPolynomialBoundary,
    pub inner: OwnedPolynomialBoundary,
    pub regions: AnnularRegionNames,
    /// Internal witness only; the exact winding proof must establish inner membership.
    pub inner_interior: [f64; 2],
    /// Selected material point, checked against every current candidate.
    pub material_seed: [f64; 2],
    pub declared_plane: Plane3,
    pub extrusion: [f64; 3],
    pub policy: OwnedAnnularPolicy,
}
impl OwnedAnnularPrismRequest {
    /// The two boundaries with their authored region-source identities, outer first.
    pub(super) fn boundaries(&self) -> [(&OwnedPolynomialBoundary, RegionSourceId); 2] {
        [
            (&self.outer, RegionSourceId(self.outer.source.0)),
            (&self.inner, RegionSourceId(self.inner.source.0)),
        ]
    }
}
