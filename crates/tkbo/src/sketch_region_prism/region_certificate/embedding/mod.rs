// SPDX-License-Identifier: GPL-2.0-only
//! ADR0029: injective affine sweep over named trimmed domains and the authored seam quotient.
mod access;
mod annular_coverage;
mod annular_regularity;
mod domains;
mod error;
mod regularity;
mod table;
mod table_validation;
#[cfg(test)]
mod table_validation_tests;
pub(in crate::sketch_region_prism) mod trim_incidence;
mod types;
use super::super::surface_charts::*;
use super::current_binding::*;
use super::*;
pub(crate) use error::*;
use rustycad_tkmath::predicates::roots::metric_bounds::*;
pub(crate) use table_validation::{PrismEmbeddingTableFailure, check_prism_embedding_table};
pub(crate) use types::*;

/// Inspect the bounded theorem; `inspect_prism_embedding(bridge,sources,receipt,options,cancel)`.
/// Successful embedding evidence leaves all mapped body diagnostics and coverage unchanged.
pub(crate) fn inspect_prism_embedding(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    requested: PrismEmbeddingOptions,
    mut cancelled: impl FnMut() -> bool,
) -> Result<PrismEmbeddingEvidence, PrismEmbeddingFailure> {
    admit(requested)?;
    let binding = capture_current_prism_binding(bridge, sources, receipt, &mut cancelled).map_err(
        |cause| {
            failure(
                requested,
                PrismEmbeddingPhase::CurrentBinding,
                None,
                cause.into(),
            )
        },
    )?;
    let evidence = construct(binding, requested, &mut cancelled)?;
    finalize(bridge, sources, receipt, &evidence, &mut cancelled)?;
    Ok(evidence)
}
fn admit(requested: PrismEmbeddingOptions) -> Result<(), PrismEmbeddingFailure> {
    if requested.version == 1 {
        return Ok(());
    }
    Err(failure(
        requested,
        PrismEmbeddingPhase::Admission,
        None,
        PrismEmbeddingCause::Version {
            actual: requested.version,
            expected: 1,
        },
    ))
}
fn construct(
    binding: CurrentPrismBinding,
    requested: PrismEmbeddingOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PrismEmbeddingEvidence, PrismEmbeddingFailure> {
    let refuse = |phase, cause| failure(requested, phase, Some(&binding), cause);
    let regularity = regularity::actual_boundary(&binding, requested.regularity, cancelled)
        .map_err(|cause| refuse(PrismEmbeddingPhase::Regularity, cause))?;
    let domains = domains::capture_domains(&binding)
        .map_err(|cause| refuse(PrismEmbeddingPhase::Incidences, cause.into()))?;
    let incidences = table::six_rows(&domains)
        .map_err(|cause| refuse(PrismEmbeddingPhase::Incidences, cause.into()))?;
    Ok(PrismEmbeddingEvidence {
        requested,
        binding,
        regularity,
        domains,
        incidences,
    })
}
fn finalize(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    evidence: &PrismEmbeddingEvidence,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), PrismEmbeddingFailure> {
    let checked = poll_prism_binding(cancelled)
        .and_then(|()| rebind_current_prism_binding(bridge, sources, receipt, &evidence.binding));
    checked.map_err(|cause| {
        failure(
            evidence.requested,
            PrismEmbeddingPhase::FinalBinding,
            Some(&evidence.binding),
            cause.into(),
        )
    })
}
#[cfg(test)]
mod tests;

#[cfg(test)]
mod trim_incidence_tests;
