// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::OriginalMemberIdentity;
use crate::sketch_region_contact_transport::ReportContactIdentity;

/// A caller assigns the semantic name before lookup; indices only verify the actual contact.
pub(crate) struct AuthoredClosureContact {
    pub name: AuthoredPrismId,
    pub actual: ReportContactIdentity,
    pub original: OriginalMemberIdentity,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct JoinedPrismTrim {
    pub occurrence: FrozenUseKey,
    pub source: AuthoredPrismId,
    pub role: PrismTrimRole,
    pub parents: Vec<FrozenOriginalKey>,
    pub interval: [f64; 2],
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct JoinedPrismFace {
    pub definition: FrozenPrismKey,
    pub occurrence: FrozenUseKey,
    pub source: AuthoredPrismId,
    pub role: crate::sketch_region_prism::PrismImageRole,
    pub parents: Vec<FrozenOriginalKey>,
    pub trims: Vec<JoinedPrismTrim>,
}
/// Multiplicities are kept before the existing frozen history set projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct JoinedParentDispositions {
    pub occurrence: FrozenUseKey,
    pub preserved: Vec<FrozenOriginalKey>,
    pub modified: Vec<FrozenOriginalKey>,
    pub generated: Vec<FrozenOriginalKey>,
}
pub(super) struct JoinedProofLinks {
    pub contact: AuthoredPrismId,
    pub original: FrozenOriginalKey,
    pub faces: Vec<JoinedPrismFace>,
    pub parents: Vec<JoinedParentDispositions>,
    pub raw_vertices: Vec<JoinedVertexOperand>,
}

/// Exact stored point and actual owner frame for one caller-authored directed use.
#[derive(Clone)]
pub(crate) struct JoinedVertexOperand {
    pub vertex: FrozenPrismKey,
    pub occurrence: FrozenUseKey,
    pub local_point: [f64; 3],
    pub placement: rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding<3>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PrismProofJoinOptions {
    pub version: u32,
    pub embedding: PrismEmbeddingOptions,
}
impl PrismProofJoinOptions {
    /// Select the exact same-body join; `PrismProofJoinOptions::v1(embedding)` retains its budgets.
    pub(crate) fn v1(embedding: PrismEmbeddingOptions) -> Self {
        Self {
            version: 1,
            embedding,
        }
    }
}
pub(crate) struct JoinedPrismProof {
    pub(super) projection: FrozenPrismProjection,
    pub(super) embedding: PrismEmbeddingEvidence,
    pub(super) links: JoinedProofLinks,
    pub(super) requested: PrismProofJoinOptions,
    pub(super) replay: Vec<u8>,
    pub(super) closure: JoinedClosureOperands,
}

/// An authored contact's directed semantic-stage operand; orientation does not infer an endpoint.
#[derive(Clone)]
pub(crate) struct JoinedClosurePointOperand {
    pub contact: AuthoredPrismId,
    pub original: FrozenOriginalKey,
    pub stage: &'static str,
    pub orientation: rustycad_tkg3d::top_abs::Orientation,
    pub local_point: [f64; 3],
    pub placement: rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding<3>,
}
pub(super) struct JoinedClosureOperands {
    pub rows: Vec<JoinedClosurePointOperand>,
    pub query: crate::sketch_region_contact_transport::ContactPointQuerySnapshot,
}
