// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrismEmbeddingPhase {
    Admission,
    CurrentBinding,
    Regularity,
    Incidences,
    FinalBinding,
}
// Private failures preserve legacy diagnostics; they are not published immutable body results.
pub(crate) struct PrismEmbeddingFailure {
    pub requested: PrismEmbeddingOptions,
    pub phase: PrismEmbeddingPhase,
    pub original: Option<Box<crate::sketch_region::OriginalMemberIdentity>>,
    pub cause: PrismEmbeddingCause,
}
pub(crate) enum PrismEmbeddingCause {
    Version {
        actual: u32,
        expected: u32,
    },
    Binding(Box<PrismBindingFailure>),
    Regularity(Box<BezierRegularityFailure>),
    NonRegular(Box<BezierRegularityReport>),
    PolynomialRequired {
        curve_id: u64,
        explicit_weights: Vec<f64>,
    },
}
pub(super) fn failure(
    requested: PrismEmbeddingOptions,
    phase: PrismEmbeddingPhase,
    binding: Option<&CurrentPrismBinding>,
    cause: PrismEmbeddingCause,
) -> PrismEmbeddingFailure {
    PrismEmbeddingFailure {
        requested,
        phase,
        original: binding.map(|b| Box::new(b.region.original.clone())),
        cause,
    }
}
impl From<PrismBindingFailure> for PrismEmbeddingCause {
    fn from(cause: PrismBindingFailure) -> Self {
        Self::Binding(Box::new(cause))
    }
}
impl From<RegionCertificateCause> for PrismEmbeddingCause {
    fn from(cause: RegionCertificateCause) -> Self {
        Self::from(PrismBindingFailure::from(cause))
    }
}
