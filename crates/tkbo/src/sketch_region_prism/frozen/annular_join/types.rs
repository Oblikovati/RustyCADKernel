// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularFrozenOptions {
    pub version: u32,
    /// Structural rows and ancestor slots, not an allocation-byte or time budget.
    pub maximum_name_units: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularNameCensus {
    pub requested: AnnularFrozenOptions,
    pub effective: AnnularFrozenOptions,
    pub measured_units: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnnularFrozenPhase {
    Admission,
    CurrentBinding,
    Capture,
    Link,
    Commit,
}
pub(in crate::sketch_region_prism) enum AnnularFrozenCause<'a> {
    AuthoredIdentity {
        quantity: &'static str,
        actual: AuthoredPrismId,
        expected: AuthoredPrismId,
    },
    Version {
        actual: u32,
        expected: u32,
    },
    NameBudget {
        maximum: usize,
        measured: Option<usize>,
    },
    Embedding(Box<AnnularEmbeddingFailure<'a>>),
    Context(Box<AnnularBindingCause<'a>>),
    Originals(Box<TwoLoopRegionFailure<'a>>),
    IncompleteCapture(Box<PrismFreezeFailure>),
    FailedCapture(Box<PrismFreezeFailure>),
    Link(Box<PrismFreezeCause>),
    Callback(rustycad_tkernel::standard::Failure),
    Cancelled {
        freeze_phase: Option<FreezePhase>,
    },
}
pub(in crate::sketch_region_prism) struct AnnularFrozenFailure<'a> {
    pub operation: AuthoredPrismId,
    pub phase: AnnularFrozenPhase,
    pub requested: AnnularFrozenOptions,
    pub cause: AnnularFrozenCause<'a>,
}
pub(super) type AnnularFrozenCheck<'a, T> = Result<T, AnnularFrozenCause<'a>>;
pub(in crate::sketch_region_prism) trait AnnularFrozenProgress:
    Send
{
    /// Poll bounded join work; `progress.checkpoint(AnnularFrozenPhase::Commit)`.
    fn checkpoint(
        &mut self,
        phase: AnnularFrozenPhase,
    ) -> Result<bool, rustycad_tkernel::standard::Failure>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct NamedAnnularWire {
    pub source: RegionSourceId,
    pub occurrence: FrozenUseKey,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum NamedAnnularDomain {
    Cap {
        outer: NamedAnnularWire,
        inner: NamedAnnularWire,
    },
    Side {
        boundary: NamedAnnularWire,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct NamedAnnularFace {
    pub chart: JoinedPrismFace,
    pub domain: NamedAnnularDomain,
}
pub(in crate::sketch_region_prism) struct AnnularFrozenDomain {
    pub(super) projection: FrozenPrismProjection,
    pub(super) embedding: AnnularEmbeddingEvidence,
    pub(super) census: AnnularNameCensus,
    pub(super) faces: Vec<NamedAnnularFace>,
    pub(super) parents: Vec<JoinedParentDispositions>,
    pub(super) raw_vertices: Vec<JoinedVertexOperand>,
    pub(super) replay: Vec<u8>,
}

impl From<PrismFreezeCause> for AnnularFrozenCause<'_> {
    fn from(cause: PrismFreezeCause) -> Self {
        Self::Link(Box::new(cause))
    }
}
