// SPDX-License-Identifier: GPL-2.0-only
use super::*;

/// Rebind and compose actual evidence; `inspect_bound_annular_embedding(request,receipt,binding,options,cancel)`.
/// All success operands are owned; current native source aliases are needed only during this call.
pub(in crate::sketch_region_prism) fn inspect_bound_annular_embedding<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    binding: AnnularPrismBinding,
    requested: AnnularEmbeddingOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<AnnularEmbeddingEvidence, Box<AnnularEmbeddingFailure<'a>>> {
    let mut phase = AnnularEmbeddingPhase::Admission;
    construct(
        request,
        receipt,
        binding,
        requested,
        &mut phase,
        &mut cancelled,
    )
    .map_err(|cause| {
        Box::new(AnnularEmbeddingFailure {
            operation: request.identities.operation,
            selected_candidate: request.identities.selected,
            phase,
            requested,
            cause,
        })
    })
}
fn construct<'a>(
    request: &TwoLoopRegionRequest<'a, '_>,
    receipt: &PrismReceipt,
    binding: AnnularPrismBinding,
    requested: AnnularEmbeddingOptions,
    phase: &mut AnnularEmbeddingPhase,
    cancelled: &mut impl FnMut() -> bool,
) -> AnnularEmbeddingCheck<'a, AnnularEmbeddingEvidence> {
    admit(requested)?;
    poll(cancelled)?;
    *phase = AnnularEmbeddingPhase::CurrentBinding;
    rebind_annular_prism_v2(request, receipt, &binding).map_err(AnnularEmbeddingCause::Binding)?;
    let evidence = propose(binding, requested, phase, cancelled)?;
    *phase = AnnularEmbeddingPhase::Commit;
    poll(cancelled)?;
    rebind_annular_prism_v2(request, receipt, &evidence.binding)
        .map_err(AnnularEmbeddingCause::Binding)?;
    Ok(evidence)
}
fn propose(
    binding: AnnularPrismBinding,
    requested: AnnularEmbeddingOptions,
    phase: &mut AnnularEmbeddingPhase,
    cancelled: &mut impl FnMut() -> bool,
) -> AnnularEmbeddingCheck<'static, AnnularEmbeddingEvidence> {
    *phase = AnnularEmbeddingPhase::Regularity;
    let operands = bound_regularity_operands(&binding)?;
    let regularity = certify_boundaries(operands, requested.regularity, cancelled)
        .map_err(AnnularEmbeddingCause::Regularity)?;
    *phase = AnnularEmbeddingPhase::Incidences;
    poll(cancelled)?;
    let domains = capture_annular_domains(&binding)?;
    let incidences = ten_rows(&domains)?;
    complete_pairs(&domains, &incidences)?;
    Ok(AnnularEmbeddingEvidence {
        requested,
        binding,
        regularity,
        domains,
        incidences,
    })
}
fn admit(requested: AnnularEmbeddingOptions) -> AnnularEmbeddingCheck<'static, ()> {
    if requested.version == 2 {
        return Ok(());
    }
    Err(AnnularEmbeddingCause::Version {
        actual: requested.version,
        expected: 2,
    })
}
fn poll(cancelled: &mut impl FnMut() -> bool) -> AnnularEmbeddingCheck<'static, ()> {
    if cancelled() {
        return Err(AnnularEmbeddingCause::Cancelled);
    }
    Ok(())
}
fn complete_pairs(
    domains: &AnnularDomains,
    incidences: &[AnnularFaceIncidence; 10],
) -> AnnularEmbeddingCheck<'static, ()> {
    let keys = [
        &domains.first,
        &domains.last,
        &domains.outer,
        &domains.inner,
    ]
    .map(|d| d.identity.clone());
    let pairs: Vec<_> = incidences
        .iter()
        .map(|row| (&row.first, &row.second))
        .collect();
    check_annular_pair_keys(&keys, &pairs).map_err(AnnularEmbeddingCause::Coverage)
}
