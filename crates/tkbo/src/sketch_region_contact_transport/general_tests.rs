// SPDX-License-Identifier: GPL-2.0-only
//! All authored analytic families enter the same source-bound transaction.
use super::test_fixture::{actual_inputs_for, contact_policy};
use super::*;
use crate::sketch_region_certification::{census_bezier_fixture, census_curved_pair_fixture};
use crate::sketch_region_partition_bindings::{binding_sources, partition_binding_sources};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

fn bridge_for(edges: Vec<Shape>) -> PairedContactTransaction {
    // WHY: every authored family exercises the same production entry and policy.
    bridge_request(edges, contact_policy())
}
fn bridge_request(edges: Vec<Shape>, policy: ContactTransportPolicy) -> PairedContactTransaction {
    let (census, mapped, edges) = actual_inputs_for(edges);
    transport_partition_contacts(census, mapped, policy, &binding_sources(&edges), |_| false)
        .unwrap_or_else(|_| panic!("expected retained general-domain transaction"))
}
fn trimmed_pair() -> Vec<Shape> {
    census_curved_pair_fixture()
        .iter()
        .map(|shape| {
            let (curve, _, _) = BRepTool::curve(&edge(shape).unwrap()).unwrap();
            BRepBuilderAPIMakeEdge::from_curve_params(&curve, 0.25, 0.75)
                .unwrap()
                .edge()
                .unwrap()
                .into_shape()
        })
        .collect()
}
fn rational_pair() -> Vec<Shape> {
    vec![
        census_bezier_fixture(&[[0., 0.], [1., 1.]], None),
        census_bezier_fixture(&[[0., 1.], [1., 0.]], Some(&[1., 2.])),
    ]
}

#[test]
fn triple_incidence_needs_independent_cross_report_correspondence() {
    let retained = bridge_for(degenerate_fixture(16));
    assert!(
        retained
            .inspection
            .contacts
            .iter()
            .all(|contact| { contact.bounded_construction.is_none() })
    );
    assert!(retained.inspection.contacts.iter().any(|contact| {
        contact.missing.iter().any(|gap| {
            matches!(gap,
            ContactTransportGap::AdditionalIncidentSources { sources, .. } if !sources.is_empty())
        })
    }));
    super::test_fixture::assert_private_incomplete(&retained);
}
#[test]
fn curved_rational_trimmed_reversed_and_permuted_pairs_use_identical_binding() {
    let mut reversed = census_curved_pair_fixture();
    reversed[0] = reversed[0].reversed();
    let mut permuted = census_curved_pair_fixture();
    permuted.reverse();
    for (name, sources) in [
        ("curved", census_curved_pair_fixture()),
        ("rational", rational_pair()),
        ("trimmed", trimmed_pair()),
        ("reversed", reversed),
        ("permuted", permuted),
    ] {
        let retained = bridge_for(sources);
        let bounded = super::test_fixture::bounded_contacts(&retained).len();
        assert_eq!(bounded, 1, "general contact family={name}");
        super::test_fixture::assert_private_incomplete(&retained);
    }
}
#[test]
fn two_distinct_algebraic_contacts_require_two_actual_shared_vertices() {
    let retained = bridge_for(partition_binding_sources(0));
    assert_eq!(retained.inspection.contacts.len(), 2);
    let bounded = retained
        .inspection
        .contacts
        .iter()
        .filter_map(|row| row.bounded_construction.as_ref())
        .collect::<Vec<_>>();
    assert_eq!(bounded.len(), 2);
    assert_ne!(bounded[0].vertex_index, bounded[1].vertex_index);
    super::test_fixture::assert_private_incomplete(&retained);
}
#[test]
fn original_shared_endpoint_keeps_modified_vertex_history_in_self_domain() {
    let retained = bridge_for(vec![crate::closed_analytic_cubic::closed_analytic_cubic(
        1., false,
    )]);
    assert_eq!(retained.inspection.contacts.len(), 1);
    let contact = &retained.inspection.contacts[0];
    assert!(contact.bounded_construction.is_some());
    assert!(
        contact
            .observed_vertices
            .iter()
            .any(|row| !row.original_parents.is_empty())
    );
    assert!(
        super::test_fixture::generated_contacts(&retained)
            .iter()
            .any(|row| !row.native_history.modified.is_empty()
                || !row.native_history.preserved.is_empty())
    );
}

pub(super) fn transport_fixture_sources(mode: usize) -> Vec<Shape> {
    match mode {
        0..=6 => partition_binding_sources(mode),
        8 => rational_pair(),
        9 => trimmed_pair(),
        12 => vec![crate::closed_analytic_cubic::closed_analytic_cubic(
            1., false,
        )],
        13 => vec![census_bezier_fixture(
            &[[3., -9.], [-1., 13.], [-1., -13.], [3., 9.]],
            None,
        )],
        7 | 10 | 11 => ordered_curved_fixture(mode),
        14..=16 => degenerate_fixture(mode),
        17 => super::declared_plane_tests::translated_sources(),
        other => panic!("contact fixture mode={other}; expected 0..18"),
    }
}
fn ordered_curved_fixture(mode: usize) -> Vec<Shape> {
    let mut sources = census_curved_pair_fixture();
    if mode == 10 {
        sources[0] = sources[0].reversed();
    }
    if mode == 11 {
        sources.reverse();
    }
    sources
}

#[test]
fn permissive_construction_limits_cannot_choose_between_two_actual_contacts() {
    let mut requested = contact_policy();
    requested.native_parameter_limits = [2.; 2];
    requested.world_endpoint_limit = 2.;
    let retained = bridge_request(partition_binding_sources(0), requested);
    assert_eq!(retained.inspection.contacts.len(), 2);
    assert!(super::test_fixture::bounded_contacts(&retained).is_empty());
    assert!(
        retained
            .inspection
            .contacts
            .iter()
            .all(|row| row.missing.iter().any(|gap| matches!(
                gap,
                ContactTransportGap::AmbiguousPairIncidence { count: 2, .. }
            )))
    );
}
#[test]
fn repeated_contact_root_remains_explicitly_unproved_with_actual_provider_evidence() {
    let retained = bridge_for(degenerate_fixture(14));
    assert_eq!(retained.inspection.contacts.len(), 1);
    let contact = &retained.inspection.contacts[0];
    assert!(contact.bounded_construction.is_none());
    assert!(contact.missing.iter().any(|gap| matches!(
        gap,
        ContactTransportGap::RepeatedRootCorrespondence {
            multiplicity: 2,
            ..
        }
    )));
}
#[test]
fn native_nearby_topology_cannot_override_complete_exact_empty_contact_domain() {
    let retained = bridge_for(degenerate_fixture(15));
    assert!(retained.inspection.contacts.is_empty());
    assert!(
        retained.inspection.missing.iter().any(|gap| matches!(gap,
        ContactTransportGap::ContactSetCardinality { expected: 0, observed, .. } if *observed > 0))
    );
    super::test_fixture::assert_private_incomplete(&retained);
}

fn degenerate_fixture(mode: usize) -> Vec<Shape> {
    assert!(
        (14..=16).contains(&mode),
        "degenerate fixture mode={mode}; expected 14..=16"
    );
    if mode == 16 {
        return triple_contact_sources();
    }
    if mode == 14 {
        return vec![
            census_bezier_fixture(&[[0., 0.], [0.5, 0.], [1., 1.]], None),
            census_bezier_fixture(&[[0., 0.], [1., 0.]], None),
        ];
    }
    vec![
        census_bezier_fixture(&[[0., 0.], [1., 0.]], None),
        census_bezier_fixture(&[[0., 1e-8], [1., 1e-8]], None),
    ]
}

fn triple_contact_sources() -> Vec<Shape> {
    let mut sources = census_curved_pair_fixture();
    sources.push(census_bezier_fixture(&[[0.5, -1.], [0.5, 1.]], None));
    sources
}
