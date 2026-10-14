// SPDX-License-Identifier: GPL-2.0-only
use super::*;
impl AnnularFrozenDomain {
    /// Borrow immutable geometry and incidence; `joined.projection()` grants no body validity.
    pub(in crate::sketch_region_prism) fn projection(&self) -> &FrozenPrismProjection {
        &self.projection
    }
    /// Borrow the complete embedding evidence; `joined.embedding()` retains both source boundaries.
    pub(in crate::sketch_region_prism) fn embedding(&self) -> &AnnularEmbeddingEvidence {
        &self.embedding
    }
    /// Borrow named certified face domains; `joined.faces()` retains directed outer/inner wires.
    pub(in crate::sketch_region_prism) fn faces(&self) -> &[NamedAnnularFace] {
        &self.faces
    }
    /// Borrow all parent-class multiplicities; `joined.parents()` grants no deletion-completeness claim.
    pub(in crate::sketch_region_prism) fn parents(&self) -> &[JoinedParentDispositions] {
        &self.parents
    }
    /// Borrow exact unrounded local vertex operands; `joined.raw_vertices()` retains owner placement.
    pub(in crate::sketch_region_prism) fn raw_vertices(&self) -> &[JoinedVertexOperand] {
        &self.raw_vertices
    }
    /// Read requested/effective units and actual count; `joined.name_census()` never widens limits.
    pub(in crate::sketch_region_prism) fn name_census(&self) -> AnnularNameCensus {
        self.census
    }
    /// Borrow complete proof-bound bytes; `joined.replay_bytes()` is independent of native addresses.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
}
impl AnnularFrozenFailure<'_> {
    /// Retain child failure classes; `failure.classification()` does not imply body invalidity.
    pub(in crate::sketch_region_prism) fn classification(&self) -> TwoLoopRefusalClass {
        match &self.cause {
            AnnularFrozenCause::Embedding(failure) => failure.classification(),
            AnnularFrozenCause::Context(cause) => cause.classification(),
            AnnularFrozenCause::Originals(failure) => failure.classification(),
            AnnularFrozenCause::FailedCapture(_) | AnnularFrozenCause::Callback(_) => {
                TwoLoopRefusalClass::Failed
            }
            AnnularFrozenCause::Link(cause) if cause.native_failure() => {
                TwoLoopRefusalClass::Failed
            }
            AnnularFrozenCause::Cancelled { .. } => TwoLoopRefusalClass::Cancelled,
            _ => TwoLoopRefusalClass::Incomplete,
        }
    }
}

impl NamedAnnularDomain {
    /// Borrow the proved boundary roles; `domain.boundary_uses()` never infers a domain from count.
    pub(in crate::sketch_region_prism) fn boundary_uses(&self) -> Vec<&FrozenUseKey> {
        match self {
            Self::Cap { outer, inner } => vec![&outer.occurrence, &inner.occurrence],
            Self::Side { boundary } => vec![&boundary.occurrence],
        }
    }
}
