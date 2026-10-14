// SPDX-License-Identifier: GPL-2.0-only
//! An owned read model remains part of the complete operation outcome, never an independent validator.
use super::{replay, rows::*};
use crate::sketch_region_prism::{
    owned_request::names::{SemanticUseKey, SingleBoundaryNamePlan, body_map::ResolvedPrismNames},
    publication::PublishedNamedPrism,
};

pub(crate) struct OwnedNamedPrismBody {
    pub(super) plan: SingleBoundaryNamePlan,
    pub(super) root: SemanticUseKey,
    pub(super) naming_binding: Vec<u8>,
    pub(super) rows: PrismBodyRows,
}
impl OwnedNamedPrismBody {
    /// Identify this read encoding; `body.version()` does not alter operation acceptance.
    pub(crate) fn version(&self) -> u32 {
        1
    }
    /// Borrow caller-qualified semantic declarations; `body.names().identities()`.
    pub(crate) fn names(&self) -> &SingleBoundaryNamePlan {
        &self.plan
    }
    /// Borrow the unique named solid root occurrence; `body.root()` is not inferred from row order.
    pub(crate) fn root(&self) -> &SemanticUseKey {
        &self.root
    }
    /// Match the complete accepted operation and naming correspondence; `body.binds(result, names)`.
    pub(crate) fn binds(&self, result: &PublishedNamedPrism, names: &ResolvedPrismNames) -> bool {
        names.binds(result)
            && self.naming_binding == names.replay_bytes()
            && &self.plan == names.plan()
    }
    /// Encode current owned rows, not a cached snapshot; `body.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> Vec<u8> {
        replay::encode(self)
    }
    /// Borrow definitions without source topology owners; `body.members()`.
    pub(crate) fn members(&self) -> &[PrismBodyMember] {
        &self.rows.members
    }
    /// Borrow complete ancestry and occurrence-local operands; `body.occurrences()`.
    pub(crate) fn occurrences(&self) -> &[PrismBodyOccurrence] {
        &self.rows.occurrences
    }
    /// Borrow actual trimmed face charts; `body.faces()`.
    pub(crate) fn faces(&self) -> &[PrismBodyFace] {
        &self.rows.faces
    }
    /// Borrow distinct first/last endpoint roles; `body.edge_endpoints()`.
    pub(crate) fn edge_endpoints(&self) -> &[PrismBodyEdgeEndpoints] {
        &self.rows.endpoints
    }
    /// Borrow each directed face-edge incidence; `body.coedges()` includes both seam uses.
    pub(crate) fn coedges(&self) -> &[PrismBodyCoedge] {
        &self.rows.coedges
    }
    /// Borrow captured cyclic wire order; `body.wire_traversals()` is separate from containment.
    pub(crate) fn wire_traversals(&self) -> &[PrismBodyWireTraversal] {
        &self.rows.wires
    }
}
