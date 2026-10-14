// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) struct AnnularEmbeddingOptions {
    pub version: u32,
    pub regularity: AnnularRegularityOptions,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum AnnularEmbeddingPhase {
    Admission,
    CurrentBinding,
    Regularity,
    Incidences,
    Commit,
}
pub(in super::super) enum AnnularEmbeddingCause<'a> {
    Version { actual: u32, expected: u32 },
    Binding(Box<AnnularBindingFailure<'a>>),
    Operands(RegionHypothesisCause),
    Regularity(Box<AnnularRegularityFailure>),
    Coverage(AnnularPairCoverageFailure<AnnularChartIdentity>),
    Cancelled,
}
pub(in super::super) struct AnnularEmbeddingFailure<'a> {
    pub operation: u64,
    pub selected_candidate: u64,
    pub phase: AnnularEmbeddingPhase,
    pub requested: AnnularEmbeddingOptions,
    pub cause: AnnularEmbeddingCause<'a>,
}
pub(in super::super) struct AnnularEmbeddingEvidence {
    pub(super) requested: AnnularEmbeddingOptions,
    pub(super) binding: AnnularPrismBinding,
    pub(super) regularity: AnnularRegularityReport,
    pub(super) domains: AnnularDomains,
    pub(super) incidences: [AnnularFaceIncidence; 10],
}
pub(super) type AnnularEmbeddingCheck<'a, T> = Result<T, AnnularEmbeddingCause<'a>>;

impl From<RegionHypothesisCause> for AnnularEmbeddingCause<'_> {
    fn from(cause: RegionHypothesisCause) -> Self {
        Self::Operands(cause)
    }
}
