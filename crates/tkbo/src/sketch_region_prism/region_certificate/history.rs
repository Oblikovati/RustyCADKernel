// SPDX-License-Identifier: GPL-2.0-only
//! Fresh history inspection and physical endpoint reuse precede mathematical domain admission.
use super::super::surface_charts::CapturedPrismSources;
use super::{PairedContactTransaction, types::*};
use crate::sketch_region::{ArrangementSource, PartitionObservation, PartitionOccurrenceIdentity};
use crate::sketch_region_certification::domain_certificate;
use crate::sketch_region_contact_transport::ReportContactIdentity;
use crate::sketch_region_partition_bindings::PartitionBindingInspection;
use crate::sketch_region_partition_bindings::{
    PartitionCaptureState, PartitionTransactionInspection, inspect_outputs,
};

pub(super) fn current_history(
    bridge: &PairedContactTransaction,
    sources: &[ArrangementSource<'_>],
    candidate: &CapturedPrismSources,
) -> RegionCheck<ReportContactIdentity> {
    let current = inspect_outputs(
        &bridge.mapped.arrangement.partition,
        bridge.mapped.copied.as_ref(),
        &bridge.census.inputs.originals,
        sources,
    )
    .map_err(RegionCertificateCause::Mapped)?;
    require_history(&current)?;
    shared_closure(bridge, &current, candidate)
}
fn require_history(current: &PartitionTransactionInspection) -> RegionCheck {
    holds(
        "complete partition capture",
        current.capture_state == PartitionCaptureState::Captured,
    )?;
    holds(
        "all current source domains",
        !current.domains.is_empty() && current.domains.iter().all(|row| row.decision.is_ok()),
    )?;
    holds(
        "all image parent bindings",
        !current.images.is_empty() && current.images.iter().all(|row| row.fault.is_none()),
    )?;
    require_candidate_history(current)
}
fn require_candidate_history(current: &PartitionTransactionInspection) -> RegionCheck {
    let generated = need(current.generated_bindings.as_ref(), "generated history")?;
    holds("observed generated history", generated.stage_observed)?;
    require("generated history faults", generated.faults.len(), 0)?;
    let history = need(current.candidate_history.as_ref(), "candidate history")?;
    holds(
        "actual candidate history available",
        history.mapped_candidates_available,
    )?;
    require("candidate history faults", history.faults.len(), 0)
}
fn shared_closure(
    bridge: &PairedContactTransaction,
    current: &PartitionTransactionInspection,
    candidate: &CapturedPrismSources,
) -> RegionCheck<ReportContactIdentity> {
    let endpoints = need(current.endpoints.as_ref(), "partition endpoints")?;
    audit_endpoint_capture(endpoints)?;
    require("one shared partition vertex", endpoints.endpoints.len(), 1)?;
    let endpoint = &endpoints.endpoints[0];
    require("two closure incidences", endpoint.incidences.len(), 2)?;
    let incidences = &endpoint.incidences;
    let parameters: Vec<_> = incidences.iter().map(|row| row.parameter).collect();
    holds(
        "partition native closure pair",
        parameters.contains(&0.) && parameters.contains(&1.),
    )?;
    contact_identity(bridge, endpoint.vertex_index, candidate)
}
fn audit_endpoint_capture(endpoints: &PartitionBindingInspection) -> RegionCheck {
    require(
        "partition structural faults",
        endpoints.structural_faults.len(),
        0,
    )?;
    require(
        "partition image span faults",
        endpoints.image_span_faults.len(),
        0,
    )?;
    holds(
        "complete native spans",
        endpoints
            .native_spans
            .iter()
            .all(|row| row.observation.is_ok()),
    )?;
    Ok(())
}

fn contact_identity(
    bridge: &PairedContactTransaction,
    vertex: i32,
    candidate: &CapturedPrismSources,
) -> RegionCheck<ReportContactIdentity> {
    require("one retained contact", bridge.inspection.contacts.len(), 1)?;
    let row = &bridge.inspection.contacts[0];
    holds("one closure domain", row.occurrence_domains == [0])?;
    bind_root_identity(bridge, row.identity)?;
    let bounded = need(row.bounded_construction.as_ref(), "bounded contact")?;
    holds(
        "retained shared identity",
        bounded.vertex_index == vertex && bounded.identity == row.identity,
    )?;
    closure_contexts(bridge, bounded)?;
    bind_generated_closure(bridge, vertex, candidate)?;
    Ok(row.identity)
}
fn bind_root_identity(
    bridge: &PairedContactTransaction,
    identity: ReportContactIdentity,
) -> RegionCheck {
    let proof = domain_certificate(&bridge.census, &bridge.census.domains[0])
        .ok_or(RegionCertificateCause::UnavailableCandidates)?;
    holds(
        "retained exact closure root",
        identity.candidate_root == proof.contact(0).expect("one closure").candidate_index,
    )?;
    let domain = &bridge.census.domains[0];
    let crate::sketch_region_certification::DomainEvidence::PairedReport { report, .. } =
        domain.evidence
    else {
        return Err(RegionCertificateCause::UnavailableCandidates);
    };
    holds("closure report identity", identity.report == report)
}

fn bind_generated_closure(
    bridge: &PairedContactTransaction,
    vertex: i32,
    candidate: &CapturedPrismSources,
) -> RegionCheck {
    let native = closure_vertex(bridge, vertex)?;
    let copy = need(bridge.mapped.copied.as_ref(), "candidate copy receipt")?;
    for (endpoint, _) in &candidate.boundaries[0].endpoints {
        holds(
            "closure copied through actual history",
            copy.links.iter().any(|link| {
                same_physical(&link.original, native) && same_physical(&link.copied, endpoint)
            }),
        )?;
    }
    Ok(())
}
pub(super) fn same_physical(
    a: &PartitionOccurrenceIdentity,
    b: &PartitionOccurrenceIdentity,
) -> bool {
    // Directed endpoint occurrences differ in orientation while reusing one located vertex.
    a.topology_address == b.topology_address && a.location == b.location
}

fn closure_contexts(
    bridge: &PairedContactTransaction,
    bounded: &crate::sketch_region_contact_transport::BoundedContactConstruction,
) -> RegionCheck {
    let copy = need(bridge.mapped.copied.as_ref(), "candidate copy receipt")?;
    let native = closure_vertex(bridge, bounded.vertex_index)?;
    let rows = crate::sketch_region_contact_transport::observe_generated_vertex(copy, native);
    let expected: Vec<_> = rows
        .iter()
        .map(|row| (row.area, row.directed_occurrence.clone()))
        .collect();
    holds(
        "current retained contact query contexts",
        super::identity_multiset::same_multiset(&expected, &bounded.directed_contexts),
    )?;
    distinct_contexts(&expected)
}
fn distinct_contexts(expected: &[(usize, PartitionOccurrenceIdentity)]) -> RegionCheck {
    let mut unique = Vec::new();
    for context in expected {
        if !unique.contains(&context) {
            unique.push(context);
        }
    }
    require("four distinct directed closure contexts", unique.len(), 4)
}

fn closure_vertex(
    bridge: &PairedContactTransaction,
    index: i32,
) -> Result<&PartitionOccurrenceIdentity, RegionCertificateCause> {
    let PartitionObservation::Captured(snapshot) = &bridge.mapped.arrangement.partition else {
        return Err(RegionCertificateCause::UnavailableCandidates);
    };
    require("one partition source", snapshot.sources.len(), 1)?;
    require("one partition vertex", snapshot.vertices.len(), 1)?;
    let matches: Vec<_> = snapshot
        .vertices
        .iter()
        .filter(|row| row.shape_index == index)
        .collect();
    require("unique current closure vertex", matches.len(), 1)?;
    Ok(&matches[0].occurrence)
}
