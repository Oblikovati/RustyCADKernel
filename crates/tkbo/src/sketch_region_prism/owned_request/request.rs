// SPDX-License-Identifier: GPL-2.0-only
//! Source-owned numerical inputs; no caller geometry handles or intermediate names.
use super::names::*;
use crate::sketch_region_certification::CensusPolicy;
use crate::sketch_region_contact_transport::ContactTransportPolicy;
use crate::sketch_region_prism::publication::PrismPublicationOptions;
use crate::sketch_region_source_conformance::SourceConformancePolicy;
use rustycad_tkmath::{gp::Plane3, predicates::roots::RootIsolationOptions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CallerSourceRevision(pub u64);
#[derive(Clone)]
pub(crate) struct OwnedPolynomialBoundary {
    pub source: CallerBoundaryId,
    pub revision: CallerSourceRevision,
    pub closure: CallerVertexId,
    pub native_start: CallerDirectedUseId,
    pub native_end: CallerDirectedUseId,
    pub world_poles: Vec<[f64; 3]>,
    pub reversed_traversal: bool,
}
#[derive(Clone, Copy)]
pub(crate) struct OwnedPrismPolicy {
    pub census: CensusPolicy,
    pub source: SourceConformancePolicy,
    pub contacts: ContactTransportPolicy,
    pub classification_roots: RootIsolationOptions,
    pub spatial_limit: f64,
    pub parameter_limit: f64,
    pub publication: PrismPublicationOptions,
}
/// Bounded V1 request; example: `request.sources.push(authored_polynomial)`.
#[derive(Clone)]
pub(crate) struct OwnedPolynomialPrismRequest {
    pub version: u32,
    pub operation: CallerOperationId,
    pub sources: Vec<OwnedPolynomialBoundary>,
    pub declared_plane: Plane3,
    pub selection: SemanticSelectionIntent,
    pub extrusion: [f64; 3],
    pub policy: OwnedPrismPolicy,
}
impl OwnedPolynomialBoundary {
    /// Borrow caller-authored identities as values; `boundary.names(operation)` proves no geometry correspondence.
    pub(in crate::sketch_region_prism) fn names(
        &self,
        operation: CallerOperationId,
    ) -> SingleBoundaryIdentities {
        SingleBoundaryIdentities {
            operation,
            boundary: self.source,
            closure: self.closure,
            start: self.native_start,
            end: self.native_end,
        }
    }
}
