// SPDX-License-Identifier: GPL-2.0-only
//! Portable body reads are separate from private proof provenance.
use super::super::{
    OwnedPolynomialPrismRequest,
    names::{body_map::*, reservations::*},
};
use crate::sketch_region_prism::{
    frozen::joined_volume::JoinedPrismVolume,
    publication::{PublishedNamedPrism, body_view::OwnedNamedPrismBody},
};
pub(crate) struct OwnedCompletePrism {
    pub(super) request: OwnedPolynomialPrismRequest,
    pub(super) publication: PublishedNamedPrism,
    pub(super) parts: CompletePrismParts,
}
pub(super) struct CompletePrismParts {
    pub names: ResolvedPrismNames,
    pub reservations: PrismReservationCoverage,
    pub body: OwnedNamedPrismBody,
    pub volume: JoinedPrismVolume,
}
impl OwnedCompletePrism {
    /// Read exact original operands and policy; `result.request().sources[0].revision`.
    pub(crate) fn request(&self) -> &OwnedPolynomialPrismRequest {
        &self.request
    }
    /// Read portable immutable geometry and incidence; `result.body().members()`.
    pub(crate) fn body(&self) -> &OwnedNamedPrismBody {
        &self.parts.body
    }
    /// Read complete actual semantic correspondence; `result.names().member_bindings()`.
    pub(crate) fn names(&self) -> &ResolvedPrismNames {
        &self.parts.names
    }
    /// Read every reservation's actual applicability; `result.reservations().rows()`.
    pub(crate) fn reservations(&self) -> &PrismReservationCoverage {
        &self.parts.reservations
    }
    /// Read positive physical volume with unchanged certified error; `result.volume().enclosure()`.
    pub(crate) fn volume(&self) -> &JoinedPrismVolume {
        &self.parts.volume
    }
    /// Borrow private validation/history/measure evidence; `result.publication().history()`.
    /// This is not a persistent application naming or body interface.
    pub(crate) fn publication(&self) -> &PublishedNamedPrism {
        &self.publication
    }
}
