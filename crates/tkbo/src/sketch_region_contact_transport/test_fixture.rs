// SPDX-License-Identifier: GPL-2.0-only
//! Actual source/copy/filler/candidate calls retain their outcomes at the bridge boundary.
use super::*;
use crate::sketch_region::*;
use crate::sketch_region_candidates::observe_candidate_copies;
use crate::sketch_region_certification::*;
use crate::sketch_region_partition_bindings::{binding_sources, single_root_edges};
use crate::sketch_region_source_conformance::*;
use rustycad_tkmath::{gp::Plane3, predicates::roots::RootIsolationOptions};

pub(super) fn contact_policy() -> ContactTransportPolicy {
    ContactTransportPolicy {
        native_parameter_limits: [1e-8; 2],
        world_endpoint_limit: 1e-8,
        roots: RootIsolationOptions::new(0.001),
    }
}
pub(super) fn actual_inputs() -> (
    BoundaryCensus,
    crate::sketch_region_partition_bindings::PartitionBoundOutputs,
    Vec<rustycad_tkbrep::topods::Shape>,
) {
    actual_inputs_for(single_root_edges())
}
pub(super) fn actual_inputs_for(
    edges: Vec<rustycad_tkbrep::topods::Shape>,
) -> (
    BoundaryCensus,
    crate::sketch_region_partition_bindings::PartitionBoundOutputs,
    Vec<rustycad_tkbrep::topods::Shape>,
) {
    let sources = binding_sources(&edges);
    let census = source_census(&sources);
    let mapped = mapped_outputs(&sources);
    (census, mapped, edges)
}
fn source_census(sources: &[ArrangementSource<'_>]) -> BoundaryCensus {
    source_census_on_plane(sources, Plane3::default())
}
pub(super) fn source_census_on_plane(
    sources: &[ArrangementSource<'_>],
    plane: Plane3,
) -> BoundaryCensus {
    let roots = contact_policy().roots;
    let inputs = capture_boundary_inputs(sources, plane)
        .unwrap_or_else(|_| panic!("expected source capture"));
    enumerate_source_bound_census(
        inputs,
        CensusPolicy {
            roots,
            max_world_error: 1e-8,
        },
        exact_fixture_source_policy(roots),
        || false,
    )
    .unwrap_or_else(|_| panic!("expected retained census"))
}
fn mapped_outputs(
    sources: &[ArrangementSource<'_>],
) -> crate::sketch_region_partition_bindings::PartitionBoundOutputs {
    mapped_outputs_on_plane(sources, Plane3::default())
}
pub(super) fn mapped_outputs_on_plane(
    sources: &[ArrangementSource<'_>],
    plane: Plane3,
) -> crate::sketch_region_partition_bindings::PartitionBoundOutputs {
    let observed = observe_partition_space(sources, &mut |_| false, capture_space_representation);
    let ArrangementCompletion::Arranged(owned) = &observed.completion else {
        panic!("expected owned arrangement");
    };
    let copied = observe_candidate_copies(
        owned,
        plane,
        &mut |_| false,
        PartitionOccurrenceIdentity::capture,
    );
    bind_observed_fixture(observed, copied, sources)
}

pub(super) fn actual_bridge() -> PairedContactTransaction {
    let (census, mapped, edges) = actual_inputs();
    transport_partition_contacts(
        census,
        mapped,
        contact_policy(),
        &binding_sources(&edges),
        |_| false,
    )
    .unwrap_or_else(|_| panic!("expected retained diagnostic transaction"))
}

pub(super) fn bounded_contacts(
    retained: &PairedContactTransaction,
) -> Vec<&super::admission::BoundedContactConstruction> {
    retained
        .inspection
        .contacts
        .iter()
        .filter_map(|contact| contact.bounded_construction.as_ref())
        .collect()
}

pub(super) fn contact_vertices(retained: &PairedContactTransaction) -> Vec<&ObservedContactVertex> {
    retained
        .inspection
        .contacts
        .iter()
        .flat_map(|contact| &contact.observed_vertices)
        .collect()
}
pub(super) fn generated_contacts(
    retained: &PairedContactTransaction,
) -> Vec<&super::generated::GeneratedContactObservation> {
    contact_vertices(retained)
        .into_iter()
        .flat_map(|vertex| &vertex.generated_observations)
        .collect()
}
pub(super) fn world_measurements(
    retained: &PairedContactTransaction,
) -> Vec<&super::world::ContactWorldMeasurement> {
    retained
        .inspection
        .contacts
        .iter()
        .flat_map(|contact| &contact.world_measurements)
        .collect()
}
pub(super) fn retry_owned_request(
    completed: PairedContactTransaction,
    edges: &[rustycad_tkbrep::topods::Shape],
) -> PairedContactTransaction {
    let PairedContactTransaction { census, mapped, .. } = completed;
    transport_partition_contacts(
        census,
        mapped,
        contact_policy(),
        &binding_sources(edges),
        |_| false,
    )
    .unwrap_or_else(|_| panic!("expected actual original-input retry"))
}
struct SecondSourcePollCancellation {
    source_polls: usize,
}
impl SecondSourcePollCancellation {
    fn cancel(&mut self, phase: ContactTransportPhase) -> bool {
        self.source_polls += usize::from(phase == ContactTransportPhase::SourceBinding);
        self.source_polls == 2
    }
}
pub(super) fn cancel_in_source_query(
    edges: &[rustycad_tkbrep::topods::Shape],
) -> ContactTransportStop {
    let (census, mapped, _) = actual_inputs_for(edges.to_vec());
    let mut cancellation = SecondSourcePollCancellation { source_polls: 0 };
    transport_partition_contacts(
        census,
        mapped,
        contact_policy(),
        &binding_sources(edges),
        |phase| cancellation.cancel(phase),
    )
    .err()
    .unwrap()
}

pub(super) fn assert_private_incomplete(retained: &PairedContactTransaction) {
    assert_eq!(
        retained.inspection.completion,
        ContactTransportCompletion::Incomplete
    );
    assert!(!retained.inspection.remaining_operation_proofs.is_empty());
}
fn exact_fixture_source_policy(roots: RootIsolationOptions) -> SourceConformancePolicy {
    SourceConformancePolicy {
        roots,
        original_plane_limit: 0.,
        construction_limit: 0.,
    }
}

fn bind_observed_fixture(
    observed: ObservedPartitionArrangement,
    copied: crate::sketch_region_candidates::CandidateCopyInspection,
    sources: &[ArrangementSource<'_>],
) -> crate::sketch_region_partition_bindings::PartitionBoundOutputs {
    let originals = capture_originals(sources).unwrap();
    crate::sketch_region_partition_bindings::bind_partition_outputs(
        observed,
        Some(copied),
        originals,
        sources,
    )
}

pub(super) fn finish_fixture_request(
    census: BoundaryCensus,
    mapped: crate::sketch_region_partition_bindings::PartitionBoundOutputs,
    sources: &[ArrangementSource<'_>],
) -> PairedContactTransaction {
    transport_partition_contacts(census, mapped, contact_policy(), sources, |_| false)
        .unwrap_or_else(|_| panic!("fixture request; expected retained transaction"))
}
