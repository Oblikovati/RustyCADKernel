// SPDX-License-Identifier: GPL-2.0-only
//! Both authored sources run through the existing census, partition, copy and contact pipelines.
use super::super::classification::{arrangement_class, candidate_class};
use super::*;
use crate::sketch_region::{
    ArrangementCompletion, ObservedPartitionArrangement, PartitionOccurrenceIdentity,
    observe_partition_space,
};
use crate::sketch_region_candidates::{CandidateCopyInspection, observe_candidate_copies};
use crate::sketch_region_certification::{capture_boundary_inputs, enumerate_source_bound_census};
use crate::sketch_region_contact_transport::{ContactTransportPhase, transport_partition_contacts};
use crate::sketch_region_partition_bindings::bind_partition_outputs;
use crate::sketch_region_source_conformance::{capture_originals, capture_space_representation};

/// Construct both closed polynomials under their authored source identities.
pub(super) fn construct(retained: &mut AnnularPreparationRetained) -> PreparationCheck<()> {
    let mut sources = Vec::new();
    for (boundary, identity) in retained.request.boundaries() {
        let source = construct_owned_polynomial_as(boundary, identity);
        sources.push(source.map_err(OwnedPreparationCause::Native)?);
    }
    retained.sources = sources;
    Ok(())
}
/// Capture both sources and enumerate the source-bound census.
pub(super) fn census(
    retained: &mut AnnularPreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> PreparationCheck<()> {
    enter(retained, callbacks, OwnedPrismPhase::SourceCapture)?;
    let sources = retained.arrangement_sources();
    let inputs = capture_boundary_inputs(&sources, retained.request.declared_plane);
    let inputs = inputs.map_err(OwnedPreparationCause::from)?;
    enter(retained, callbacks, OwnedPrismPhase::Census)?;
    let policy = retained.request.policy;
    let poll = || callbacks.poll(OwnedPrismPhase::Census);
    let census = enumerate_source_bound_census(inputs, policy.census, policy.source, poll);
    retained.census = Some(census.map_err(OwnedPreparationCause::from)?);
    Ok(())
}
/// Arrange both sources, copy candidates and bind partition outputs to the original owners.
pub(super) fn partition(
    retained: &mut AnnularPreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> PreparationCheck<()> {
    let phase = OwnedPrismPhase::Partition(crate::sketch_region::ArrangementPhase::Admission);
    enter(retained, callbacks, phase)?;
    let sources = retained.arrangement_sources();
    let originals = capture_originals(&sources).map_err(OwnedPreparationCause::Native)?;
    let poll = &mut |p| callbacks.poll(OwnedPrismPhase::Partition(p));
    let observed = observe_partition_space(&sources, poll, capture_space_representation);
    let copied = candidates(&observed, retained.request.declared_plane, callbacks);
    let bound = bind_partition_outputs(observed, copied, originals, &sources);
    retained.partition = Some(bound);
    require_candidates(retained)
}
fn candidates(
    observed: &ObservedPartitionArrangement,
    plane: rustycad_tkmath::gp::Plane3,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> Option<CandidateCopyInspection> {
    let ArrangementCompletion::Arranged(owned) = &observed.completion else {
        return None;
    };
    let poll = &mut |p| callbacks.poll(OwnedPrismPhase::Candidate(p));
    Some(observe_candidate_copies(
        owned,
        plane,
        poll,
        PartitionOccurrenceIdentity::capture,
    ))
}
fn require_candidates(retained: &AnnularPreparationRetained) -> PreparationCheck<()> {
    let partition = local(retained.partition.as_ref(), "bound partition")?;
    let completion = &partition.arrangement.completion;
    if !matches!(completion, ArrangementCompletion::Arranged(_)) {
        let class = arrangement_class(completion);
        return Err(OwnedPreparationCause::StageUnavailable { class }.into());
    }
    let copied = local(
        partition.copied.as_ref(),
        "actual candidate copy observation",
    )?;
    let unavailable = |cause| OwnedPreparationCause::StageUnavailable {
        class: candidate_class(cause),
    };
    copied
        .completion
        .as_ref()
        .map(|_| ())
        .map_err(|c| unavailable(c).into())
}
/// Transport partition contacts for both sources into one paired transaction.
pub(super) fn contacts(
    retained: &mut AnnularPreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
) -> PreparationCheck<()> {
    let phase = OwnedPrismPhase::Contact(ContactTransportPhase::SourceBinding);
    enter(retained, callbacks, phase)?;
    let census = local(retained.census.take(), "census")?;
    let partition = local(retained.partition.take(), "partition")?;
    let sources = retained.arrangement_sources();
    let policy = retained.request.policy.contacts;
    let poll = |p| callbacks.poll(OwnedPrismPhase::Contact(p));
    let bridge = transport_partition_contacts(census, partition, policy, &sources, poll);
    retained.bridge = Some(bridge.map_err(OwnedPreparationCause::Contact)?);
    Ok(())
}
/// Enter one phase; a recorded callback stop is terminal.
pub(super) fn enter(
    retained: &mut AnnularPreparationRetained,
    callbacks: &mut OwnedPrismCallbacks<'_>,
    phase: OwnedPrismPhase,
) -> PreparationCheck<()> {
    retained.phase = phase;
    if callbacks.poll(phase) {
        return Err(OwnedPreparationCause::Callback.into());
    }
    Ok(())
}
