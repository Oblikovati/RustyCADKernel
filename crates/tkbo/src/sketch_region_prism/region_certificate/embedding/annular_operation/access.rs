// SPDX-License-Identifier: GPL-2.0-only
use super::*;

impl AnnularEmbeddingEvidence {
    /// Borrow exact proof bytes; `evidence.replay_bytes()` retains all four domain claims.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
    /// Borrow the actual source binding; `evidence.binding()` grants no mutation.
    pub(in crate::sketch_region_prism) fn binding(&self) -> &AnnularPrismBinding {
        &self.binding
    }
    /// Borrow all four certified domains; `evidence.domains()` includes both annular caps.
    pub(in crate::sketch_region_prism) fn domains(&self) -> &AnnularDomains {
        &self.domains
    }
    /// Borrow every unordered face-pair claim; `evidence.incidences()` includes the diagonal.
    pub(in crate::sketch_region_prism) fn incidences(&self) -> &[AnnularFaceIncidence; 10] {
        &self.incidences
    }
    /// Borrow both numerical certificates; `evidence.regularity()` retains effective budgets.
    pub(in crate::sketch_region_prism) fn regularity(&self) -> &AnnularRegularityReport {
        &self.regularity
    }
    /// Return the unchanged caller policy; `evidence.requested()` retains behavior version.
    pub(in crate::sketch_region_prism) fn requested(&self) -> AnnularEmbeddingOptions {
        self.requested
    }
}
/// Recheck live binding without callbacks; `rebind_bound_annular_embedding(request,receipt,evidence)`.
/// The sealed owned proof remains immutable; a successful rebind grants no ordered body validity.
pub(in crate::sketch_region_prism) fn rebind_bound_annular_embedding<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    evidence: &AnnularEmbeddingEvidence,
) -> Result<(), Box<AnnularEmbeddingFailure<'a>>> {
    rebind_annular_prism_v2(request, receipt, evidence.binding()).map_err(|cause| {
        Box::new(AnnularEmbeddingFailure {
            operation: request.identities.operation,
            selected_candidate: request.identities.selected,
            phase: AnnularEmbeddingPhase::CurrentBinding,
            requested: evidence.requested(),
            cause: AnnularEmbeddingCause::Binding(cause),
        })
    })
}
