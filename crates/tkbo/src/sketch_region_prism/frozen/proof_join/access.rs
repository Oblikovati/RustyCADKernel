// SPDX-License-Identifier: GPL-2.0-only
use super::*;
impl JoinedPrismProof {
    /// Read the caller-authored contact name; `join.closure_contact()` never returns a report rank.
    pub(crate) fn closure_contact(&self) -> AuthoredPrismId {
        self.links.contact
    }
    /// Borrow the authored original boundary name; `join.original()` retains member and directed use.
    pub(crate) fn original(&self) -> &FrozenOriginalKey {
        &self.links.original
    }

    /// Borrow all raw semantic closure uses; `join.raw_closure_points()` keeps twelve directed contexts.
    pub(crate) fn raw_closure_points(&self) -> &[JoinedClosurePointOperand] {
        &self.closure.rows
    }
    /// Borrow the original singleton query operands; `join.closure_query()` never substitutes a candidate curve.
    pub(crate) fn closure_query(
        &self,
    ) -> &crate::sketch_region_contact_transport::ContactPointQuerySnapshot {
        &self.closure.query
    }

    /// Borrow this exact immutable projection; `join.projection()` still grants no body validity.
    pub(crate) fn projection(&self) -> &FrozenPrismProjection {
        &self.projection
    }
    /// Borrow the actual-source theorem; `join.embedding().binding()` retains every current chart operand.
    pub(crate) fn embedding(&self) -> &PrismEmbeddingEvidence {
        &self.embedding
    }
    /// Borrow exact local vertex operands; `join.raw_vertices()` includes every authored directed owner.
    pub(crate) fn raw_vertices(&self) -> &[JoinedVertexOperand] {
        &self.links.raw_vertices
    }
    /// Borrow exact authored face/trim links; `join.faces()` never names by returned rank.
    pub(crate) fn faces(&self) -> &[JoinedPrismFace] {
        &self.links.faces
    }
    /// Borrow complete output parent multiplicities; `join.parent_dispositions()` precedes set projection.
    pub(crate) fn parent_dispositions(&self) -> &[JoinedParentDispositions] {
        &self.links.parents
    }
    /// Borrow the explicit work policy; `join.requested()` includes embedding root budgets.
    pub(crate) fn requested(&self) -> PrismProofJoinOptions {
        self.requested
    }
    /// Borrow complete non-address bytes; downstream certificates bind to `join.replay_bytes()`.
    pub(crate) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
}
