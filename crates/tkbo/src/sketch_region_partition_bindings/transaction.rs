// SPDX-License-Identifier: GPL-2.0-only
//! Retain original operation causes together with opt-in owned binding diagnostics.
use super::{
    candidate_history::inspect_candidate_history, history_types::CandidateHistoryInspection,
};
use super::{image_links::*, source_domains::*, *};
use crate::sketch_region::{ArrangementSource, ObservedPartitionArrangement, PartitionObservation};
use crate::sketch_region_candidates::CandidateCopyInspection;
use crate::sketch_region_source_conformance::{OriginalCurveCapture, capture_originals};
use rustycad_tkernel::standard::Failure;

pub(crate) struct PartitionBoundOutputs {
    pub arrangement: ObservedPartitionArrangement,
    pub copied: Option<CandidateCopyInspection>,
    pub originals: Vec<OriginalCurveCapture>,
    pub inspection: Result<PartitionTransactionInspection, Failure>,
}
pub(crate) struct PartitionTransactionInspection {
    pub capture_state: PartitionCaptureState,
    pub endpoints: Option<PartitionBindingInspection>,
    pub images: Vec<PartitionImageBinding>,
    pub domains: Vec<PartitionSourceDomainBinding>,
    pub generated_bindings: Option<super::generated_bindings::GeneratedBindingInspection>,
    pub candidate_history: Option<CandidateHistoryInspection>,
    pub missing: Vec<RequiredPartitionProof>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PartitionCaptureState {
    NotReached,
    Partial,
    Captured,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RequiredPartitionProof {
    PairedContactIdentity,
    NativeDualParameterTransport,
    WorldEndpointBound,
    CandidateSearchCoverage,
    OrderedBodyValidation,
}
/// Inspect actual operation outputs; this cannot return a healthy region or body.
/// Example: `bind_partition_outputs(observed, copied, originals, &current_sources)`.
pub(crate) fn bind_partition_outputs(
    arrangement: ObservedPartitionArrangement,
    copied: Option<CandidateCopyInspection>,
    originals: Vec<OriginalCurveCapture>,
    current_sources: &[ArrangementSource<'_>],
) -> PartitionBoundOutputs {
    let inspection = inspect_outputs(
        &arrangement.partition,
        copied.as_ref(),
        &originals,
        current_sources,
    );
    PartitionBoundOutputs {
        arrangement,
        copied,
        originals,
        inspection,
    }
}
fn inspect_outputs(
    observation: &PartitionObservation,
    copied: Option<&CandidateCopyInspection>,
    originals: &[OriginalCurveCapture],
    current_sources: &[ArrangementSource<'_>],
) -> Result<PartitionTransactionInspection, Failure> {
    let mut inspection = empty_transaction(observation);
    let Some(snapshot) = available_snapshot(observation) else {
        return Ok(inspection);
    };
    let current = capture_originals(current_sources)?;
    inspection.endpoints = Some(inspect_partition_bindings(snapshot));
    inspection.domains = inspect_source_domains(snapshot, originals, &current);
    attach_copy_inspections(snapshot, copied, &mut inspection);
    Ok(inspection)
}
fn available_snapshot(
    observation: &PartitionObservation,
) -> Option<&crate::sketch_region::OwnedPartitionSnapshot> {
    match observation {
        PartitionObservation::NotReached => None,
        PartitionObservation::Captured(snapshot) => Some(snapshot),
        PartitionObservation::Incomplete { partial, .. } => Some(partial),
    }
}
fn empty_transaction(observation: &PartitionObservation) -> PartitionTransactionInspection {
    let capture_state = match observation {
        PartitionObservation::NotReached => PartitionCaptureState::NotReached,
        PartitionObservation::Captured(_) => PartitionCaptureState::Captured,
        PartitionObservation::Incomplete { .. } => PartitionCaptureState::Partial,
    };
    PartitionTransactionInspection {
        capture_state,
        endpoints: None,
        images: Vec::new(),
        domains: Vec::new(),
        candidate_history: None,
        generated_bindings: None,
        missing: required_partition_proofs(),
    }
}
fn required_partition_proofs() -> Vec<RequiredPartitionProof> {
    vec![
        RequiredPartitionProof::PairedContactIdentity,
        RequiredPartitionProof::NativeDualParameterTransport,
        RequiredPartitionProof::WorldEndpointBound,
        RequiredPartitionProof::CandidateSearchCoverage,
        RequiredPartitionProof::OrderedBodyValidation,
    ]
}

fn attach_copy_inspections(
    snapshot: &crate::sketch_region::OwnedPartitionSnapshot,
    copied: Option<&CandidateCopyInspection>,
    inspection: &mut PartitionTransactionInspection,
) {
    let Some(copied) = copied else {
        return;
    };
    inspection.images = inspect_image_links(snapshot, &copied.links);
    inspection.generated_bindings = Some(super::generated_bindings::inspect_generated_bindings(
        copied,
    ));
    inspection.candidate_history = Some(inspect_candidate_history(copied));
}
