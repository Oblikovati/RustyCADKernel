// SPDX-License-Identifier: GPL-2.0-only
//! Inspect only borrowed diagnostic geometry; no path changes receipt coverage or publishes a body.
use super::super::{PrismReceipt, region_certificate::current_binding::*};
use super::{
    census::capture_census, census_types::PrismToleranceCensus, inspection_types::*,
    metadata::capture_metadata, policy::*, rebind::rebind_local_capture, residuals::*,
};
use crate::sketch_region::ArrangementSource;
use crate::sketch_region_contact_transport::PairedContactTransaction;

/// Capture explicit metadata and residual evidence; example: `inspect_prism_tolerances(bridge,sources,receipt,request,progress)`.
/// Unresolved placement semantics always prevent an aggregate tolerance certification.
pub(crate) fn inspect_prism_tolerances(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    receipt: &PrismReceipt,
    request: PrismToleranceRequest,
    progress: &mut impl ToleranceInspectionProgress,
) -> InspectionCheck<PrismToleranceInspection> {
    let binding = capture_current_prism_binding(bridge, sources, receipt, &mut || {
        progress.cancelled(ToleranceInspectionPhase::Capture)
    })
    .map_err(|cause| binding_failure(ToleranceInspectionPhase::Capture, cause))?;
    let inspection = inspect_bound(receipt, binding, request, progress)?;
    poll(progress, ToleranceInspectionPhase::Commit)?;
    rebind_current_prism_binding(bridge, sources, receipt, &inspection.binding)
        .map_err(|cause| binding_failure(ToleranceInspectionPhase::Commit, cause))?;
    rebind_local_capture(receipt, &inspection)?;
    Ok(inspection)
}
fn inspect_bound(
    receipt: &PrismReceipt,
    binding: CurrentPrismBinding,
    request: PrismToleranceRequest,
    progress: &mut impl ToleranceInspectionProgress,
) -> InspectionCheck<PrismToleranceInspection> {
    let census = capture_census(receipt, &binding)
        .map_err(|cause| census_failure(ToleranceInspectionPhase::Capture, cause))?;
    poll(progress, ToleranceInspectionPhase::Metadata)?;
    let metadata = capture_metadata(receipt, &census, request)?;
    let endpoints = endpoint_evidence(&census, request, progress)?;
    Ok(assembled(binding, census, metadata, endpoints, request))
}
fn assembled(
    binding: CurrentPrismBinding,
    census: PrismToleranceCensus,
    metadata: Vec<ToleranceMetadataEvidence>,
    endpoints: Vec<ToleranceEndpointEvidence>,
    request: PrismToleranceRequest,
) -> PrismToleranceInspection {
    let trims = trim_evidence(&census);
    PrismToleranceInspection {
        behavior_version: 1,
        requested: request,
        binding,
        census,
        metadata,
        endpoints,
        trims,
        unresolved: vec![ToleranceRelationGap::StoredTolerancePlacementSemantics],
    }
}
