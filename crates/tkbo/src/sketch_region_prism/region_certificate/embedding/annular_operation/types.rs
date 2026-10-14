// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularEmbeddingOptions {
    pub version: u32,
    pub regularity: AnnularRegularityOptions,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) enum AnnularEmbeddingPhase {
    Admission,
    CurrentBinding,
    Regularity,
    Incidences,
    Commit,
}
pub(in crate::sketch_region_prism) enum AnnularEmbeddingCause<'a> {
    Version { actual: u32, expected: u32 },
    Binding(Box<AnnularBindingFailure<'a>>),
    Operands(RegionHypothesisCause),
    Regularity(Box<AnnularRegularityFailure>),
    Coverage(AnnularPairCoverageFailure<AnnularChartIdentity>),
    Encoding(Box<AnnularBindingCause<'a>>),
    Cancelled,
}
pub(in crate::sketch_region_prism) struct AnnularEmbeddingFailure<'a> {
    pub operation: u64,
    pub selected_candidate: u64,
    pub phase: AnnularEmbeddingPhase,
    pub requested: AnnularEmbeddingOptions,
    pub cause: AnnularEmbeddingCause<'a>,
}
pub(in crate::sketch_region_prism) struct AnnularEmbeddingEvidence {
    pub(super) replay: Vec<u8>,
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
