// SPDX-License-Identifier: GPL-2.0-only
//! Opt-in borrowed entry; original receipts are never moved from or promoted to completion.
mod boundary;
mod closure_points;
pub(crate) use closure_points::current_closure_vertices;
mod closure_types;
mod identity_multiset;
pub(crate) use closure_types::*;
mod containment;
mod history;
mod inspect;
mod rebind;
mod source;
mod types;
use super::*;
use crate::sketch_region::ArrangementSource;
use crate::sketch_region_candidates::*;
use crate::sketch_region_contact_transport::PairedContactTransaction;
pub(crate) use types::*;

/// Certify current source/copy hypotheses after the general transaction; `retain_source_bound_prism(&bridge,&sources,request,progress)`.
pub(crate) fn retain_source_bound_prism(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    request: PrismRequest,
    progress: &mut dyn PrismProgress,
) -> Result<PrismReceipt, Box<RegionPrismUnavailable>> {
    let batch = borrowed_batch(bridge).ok_or_else(|| unavailable(request))?;
    let mut receipt = retain_selected_prism(temporary_view(batch), request, progress);
    restore_gaps(&mut receipt);
    if matches!(receipt.completion, PrismCompletion::Incomplete) {
        retain_certificate(bridge, sources, &mut receipt);
    }
    Ok(receipt)
}
fn retain_certificate(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &mut PrismReceipt,
) {
    restore_gaps(receipt);
    let decision = inspect::inspect(bridge, sources, receipt);
    if decision.is_ok() {
        discharge_gaps(receipt);
    }
    receipt.region = Some(RegionCertificateInspection {
        version: 1,
        decision,
    });
}

fn borrowed_batch(bridge: &PairedContactTransaction) -> Option<&CandidateBatch> {
    let copied = bridge.mapped.copied.as_ref()?;
    // Borrow the actual outcome; absence never fabricates an empty successful batch.
    copied.completion.as_ref().ok()
}
fn temporary_view(batch: &CandidateBatch) -> CandidateBatch {
    CandidateBatch {
        plane: batch.plane,
        candidates: batch.candidates.iter().map(temporary_candidate).collect(),
        input_deleted: batch.input_deleted.clone(),
        unrepresented: batch.unrepresented.clone(),
        mapped_report: std::sync::Arc::clone(&batch.mapped_report),
    }
}
fn temporary_candidate(candidate: &RegionCandidate) -> RegionCandidate {
    // These aliases only supply the existing pipeline. Its explicit deep copy owns sweep geometry.
    RegionCandidate {
        face: candidate.face.clone(),
        wires: candidate.wires.clone(),
        relations: candidate.relations.clone(),
    }
}
fn unavailable(requested: PrismRequest) -> Box<RegionPrismUnavailable> {
    Box::new(RegionPrismUnavailable {
        requested,
        cause: RegionCertificateCause::UnavailableCandidates,
        missing: obligations().to_vec(),
    })
}
fn obligations() -> [MissingCertificate; 4] {
    use MissingCertificate::*;
    [
        Construction,
        SharedIncidenceReuse,
        CandidateHistoryCoverage,
        SearchCompleteness,
    ]
}
fn restore_gaps(receipt: &mut PrismReceipt) {
    for gap in obligations() {
        if !receipt
            .gaps
            .iter()
            .any(|row| matches!(row,PrismGap::Candidate(value) if *value==gap))
        {
            receipt.gaps.push(PrismGap::Candidate(gap));
        }
    }
    if let Some(CandidateClassification::Incomplete { missing, .. }) = &mut receipt.classification {
        for gap in obligations() {
            if !missing.contains(&gap) {
                missing.push(gap);
            }
        }
    }
}
fn discharge_gaps(receipt: &mut PrismReceipt) {
    receipt
        .gaps
        .retain(|row| !matches!(row,PrismGap::Candidate(gap) if obligations().contains(gap)));
    if let Some(CandidateClassification::Incomplete { missing, .. }) = &mut receipt.classification {
        missing.retain(|gap| !obligations().contains(gap));
    }
}
#[cfg(test)]
mod callback_tests;
#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod evaluator_tests;
#[cfg(test)]
mod isolation_tests;
#[cfg(test)]
mod rejection_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
pub(in crate::sketch_region_prism) mod tests;

#[cfg(test)]
mod point_tests;

pub(crate) mod current_binding;
pub(in crate::sketch_region_prism) mod measurements;
#[allow(unused_imports)]
pub(crate) use measurements::{
    PrismMeasureInspection, PrismMeasureOptions, inspect_prism_measurements,
};
#[cfg(test)]
mod embedding_baseline_tests;

pub(crate) mod embedding;
pub(crate) mod orientation;

#[allow(unused_imports)]
pub(crate) use measurements::{
    CompletePrismMeasureInspection, CompletePrismMeasureOptions,
    inspect_complete_prism_measurements,
};

pub(crate) use measurements::{
    BoundPrismMeasureFailure, CompletePrismMeasures, PrismScalarFailure, measure_bound_prism,
};
