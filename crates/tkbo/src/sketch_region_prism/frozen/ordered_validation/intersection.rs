// SPDX-License-Identifier: GPL-2.0-only
//! The same-body theorem table is retained after exact correspondence checking, never recomputed.
use super::*;
use crate::sketch_region_prism::region_certificate::embedding::{
    PrismFaceDomain, PrismFaceIncidence,
};

pub(crate) struct JoinedPrismIntersectionCertificate {
    pub(super) source: Box<[u8]>,
    pub(super) domains: [PrismFaceDomain; 3],
    pub(super) incidences: [PrismFaceIncidence; 6],
}
impl JoinedPrismIntersectionCertificate {
    /// Match full joined operands, not only face counts; example: `proof.binds(&join)`.
    pub(crate) fn binds(&self, join: &JoinedPrismProof) -> bool {
        let expected = join.replay_bytes();
        self.source.as_ref() == expected
    }
    /// Borrow complete typed domains and all trim occurrences; example: `proof.domains()`.
    pub(crate) fn domains(&self) -> &[PrismFaceDomain; 3] {
        // A cap domain is its Jordan region, not its unbounded support plane.
        &self.domains
    }
    /// Borrow three self and three unordered pair conclusions; example: `proof.incidences()`.
    pub(crate) fn incidences(&self) -> &[PrismFaceIncidence; 6] {
        // The same-face seam requires both distinct authored occurrences.
        &self.incidences
    }
    pub(super) fn checked(join: &JoinedPrismProof) -> Self {
        Self {
            source: join.replay_bytes().into(),
            domains: join.embedding().domains().clone(),
            incidences: join.embedding().incidences().clone(),
        }
    }
}
