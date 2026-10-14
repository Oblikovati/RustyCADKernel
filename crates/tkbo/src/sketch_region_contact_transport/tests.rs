// SPDX-License-Identifier: GPL-2.0-only
//! Actual source/copy/filler/candidate calls retain their outcomes at the bridge boundary.
use super::*;
use crate::sketch_region_partition_bindings::{binding_sources, single_root_edges};
use crate::sketch_region_source_conformance::*;
use rustycad_tkmath::predicates::roots::metric_bounds::*;

use super::test_fixture::*;
#[test]
fn actual_sources_are_independently_proved_exact_without_promoting_region() {
    let retained = actual_bridge();
    assert!(!retained.exact_sources.is_empty());
    assert!(
        retained
            .exact_sources
            .iter()
            .all(|row| matches!(row.decision, Ok(ResidualDecision::Within { .. })))
    );
    super::test_fixture::assert_private_incomplete(&retained);
    assert!(!retained.inspection.remaining_operation_proofs.is_empty());
}
#[test]
fn actual_shared_vertex_keeps_native_approximation_separate_from_exact_root() {
    let retained = actual_bridge();
    let rows = contact_vertices(&retained);
    assert!(!rows.is_empty());
    assert!(rows.iter().any(|row| row.native_parameters[0] != 0.5));
    assert!(rows.iter().any(|row| matches!(&row.parameter_measurement, Some(Ok(Some(axes)))
        if axes.iter().all(|axis| axis.decision() == rustycad_tkmath::predicates::roots::bezier_pair::PairParameterDecision::Within))));
    assert!(
        retained
            .inspection
            .contacts
            .iter()
            .all(|row| row.bounded_construction.is_some() && row.missing.is_empty())
    );
    super::test_fixture::assert_private_incomplete(&retained);
}
#[test]
fn policy_failure_retains_owned_original_outcomes() {
    let (census, mapped, edges) = actual_inputs();
    let mut policy = contact_policy();
    policy.native_parameter_limits[1] = -1.;
    let failed =
        transport_partition_contacts(census, mapped, policy, &binding_sources(&edges), |_| false);
    let Err(ContactTransportStop::Policy {
        field,
        value,
        completed,
    }) = failed
    else {
        panic!("expected typed policy failure");
    };
    assert_eq!((field, value), ("native u parameter limit", -1.));
    assert!(completed.mapped.copied.is_some());
    assert!(completed.exact_sources.is_empty());
}
#[test]
fn phase_cancellation_retains_original_completed_source_proofs() {
    let (census, mapped, edges) = actual_inputs();
    let cancelled = transport_partition_contacts(
        census,
        mapped,
        contact_policy(),
        &binding_sources(&edges),
        |phase| phase == ContactTransportPhase::TopologyBinding,
    );
    let Err(ContactTransportStop::Cancelled { phase, completed }) = cancelled else {
        panic!("expected retained cancellation");
    };
    assert_eq!(phase, ContactTransportPhase::TopologyBinding);
    assert!(!completed.exact_sources.is_empty());
}

#[test]
fn actual_directed_generated_points_keep_copy_identity_and_typed_candidate_failure() {
    let retained = actual_bridge();
    let copied = retained.mapped.copied.as_ref().unwrap();
    assert!(copied.completion.is_err());
    let observations = generated_contacts(&retained);
    assert!(!observations.is_empty());
    assert!(
        observations
            .iter()
            .all(|row| row.native_image.topology_address != row.copied_image.topology_address)
    );
    assert!(
        observations
            .iter()
            .all(|row| row.copied_point.as_ref().unwrap().is_ok()
                && row.directed_point.as_ref().unwrap().is_ok()
                && !row.copied_history.generated.is_empty())
    );
}
#[test]
fn original_exact_contact_bracket_bounds_actual_world_point_without_native_snap() {
    let retained = actual_bridge();
    let rows = world_measurements(&retained);
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|row| matches!(&row.report, Ok(report)
        if matches!(report.decision, Ok(ResidualDecision::Within { .. })))));
    assert!(rows.iter().all(|row| {
        row.original_bounds
            .iter()
            .all(|bound| bound.compare_f64(0.5).unwrap().is_eq())
    }));
    assert!(rows.iter().any(|row| row.point != [0.5, 0.25, 0.]));
}

#[test]
fn zero_limits_reject_native_construction_error_without_changing_certified_root() {
    let (census, mapped, edges) = actual_inputs();
    let mut policy = contact_policy();
    policy.native_parameter_limits = [0.; 2];
    policy.world_endpoint_limit = 0.;
    let retained =
        transport_partition_contacts(census, mapped, policy, &binding_sources(&edges), |_| false)
            .unwrap_or_else(|_| panic!("expected retained construction diagnostics"));
    let contacts = &retained.inspection.contacts;
    assert!(contacts.iter().flat_map(|row| &row.observed_vertices).any(|row|
        matches!(&row.parameter_measurement, Some(Ok(Some(axes))) if axes.iter().any(|axis|
            axis.decision() == rustycad_tkmath::predicates::roots::bezier_pair::PairParameterDecision::Exceeds))));
    assert!(contacts.iter().flat_map(|row| &row.world_measurements).any(|row|
        matches!(&row.report, Ok(report) if matches!(report.decision, Ok(ResidualDecision::Exceeds { .. })))));
    assert_original_half_world_brackets(contacts);
}

#[test]
fn in_solver_cancellation_retains_typed_prefix_and_retries_original_owned_inputs() {
    let edges = single_root_edges();
    let stopped = cancel_in_source_query(&edges);
    let ContactTransportStop::Cancelled { phase, completed } = stopped else {
        panic!("expected latched in-solver cancellation");
    };
    assert_eq!(phase, ContactTransportPhase::SourceBinding);
    assert_cancelled_source_prefix(&completed);
    let retried = retry_owned_request(*completed, &edges);
    assert!(!retried.inspection.contacts.is_empty());
    super::test_fixture::assert_private_incomplete(&retried);
}

#[test]
fn reversed_alias_identity_queries_reuse_only_actual_full_scalar_source_inputs() {
    let mut edges = single_root_edges();
    edges.push(edges[0].reversed());
    let (census, mapped, edges) = actual_inputs_for(edges);
    let retained = transport_partition_contacts(
        census,
        mapped,
        contact_policy(),
        &binding_sources(&edges),
        |_| false,
    )
    .unwrap_or_else(|_| panic!("expected retained alias transaction"));
    assert_reused_source_proofs(&retained);
}

#[test]
fn actual_unique_contact_has_bounded_local_construction_without_candidate_or_body_success() {
    let retained = actual_bridge();
    let bounded = bounded_contacts(&retained);
    assert_eq!(bounded.len(), 1);
    assert!(!bounded[0].directed_contexts.is_empty());
    assert!(retained.mapped.copied.as_ref().unwrap().completion.is_err());
    super::test_fixture::assert_private_incomplete(&retained);
    assert!(!retained.inspection.remaining_operation_proofs.is_empty());
    let measurements = world_measurements(&retained);
    let measurement = measurements[0];
    assert!(measurement.roles.len() >= 3);
}

#[test]
fn missing_native_generated_history_cannot_be_replaced_by_near_points_and_parent_membership() {
    let mut retained = actual_bridge();
    assert!(!super::test_fixture::bounded_contacts(&retained).is_empty());
    retained
        .inspection
        .contacts
        .iter_mut()
        .flat_map(|contact| &mut contact.observed_vertices)
        .flat_map(|vertex| &mut vertex.generated_observations)
        .for_each(|observed| observed.native_history.generated.clear());
    super::admission::inspect_contact_admission(&mut retained);
    assert!(super::test_fixture::bounded_contacts(&retained).is_empty());
}
#[test]
fn missing_actual_shared_vertex_never_admits_from_retained_metric_evidence() {
    let mut retained = actual_bridge();
    assert!(
        !retained.inspection.contacts[0]
            .world_measurements
            .is_empty()
    );
    for contact in &mut retained.inspection.contacts {
        contact.observed_vertices.clear();
    }
    super::admission::inspect_contact_admission(&mut retained);
    assert!(super::test_fixture::bounded_contacts(&retained).is_empty());
    assert!(retained.inspection.contacts.iter().all(|row| {
        row.missing
            .iter()
            .any(|gap| matches!(gap, ContactTransportGap::MissingActualPairIncidence { .. }))
    }));
}

fn assert_cancelled_source_prefix(completed: &PairedContactTransaction) {
    assert_eq!(completed.exact_sources.len(), 1);
    assert!(matches!(
        completed.exact_sources[0].decision,
        Err(SourceConformanceCause::Exact(ResidualError::Roots(
            rustycad_tkmath::predicates::roots::RootIsolationError::Cancelled
        )))
    ));
}

fn assert_reused_source_proofs(retained: &PairedContactTransaction) {
    let reused = retained
        .exact_sources
        .iter()
        .filter(|row| row.reused_certificate.is_some())
        .collect::<Vec<_>>();
    assert!(!reused.is_empty());
    for row in reused {
        let original = &retained.exact_sources[row.reused_certificate.unwrap()];
        assert!(original.reused_certificate.is_none());
        assert_eq!(row.completed_bisections, 0);
        assert_eq!(row.certificate_bisections, original.completed_bisections);
        assert!(row.original != original.original || row.occurrence != original.occurrence);
    }
}

fn assert_original_half_world_brackets(contacts: &[ObservedPairedContact]) {
    assert!(
        contacts
            .iter()
            .flat_map(|row| &row.world_measurements)
            .all(|row| row
                .original_bounds
                .iter()
                .all(|bound| bound.compare_f64(0.5).unwrap().is_eq()))
    );
}
