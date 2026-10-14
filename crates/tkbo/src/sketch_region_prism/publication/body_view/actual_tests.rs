// SPDX-License-Identifier: GPL-2.0-only
//! Actual sealed resolver/publication inputs; no test constructor supplies body authority.
use super::*;
use crate::sketch_region_prism::owned_request::*;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricCurve;

pub(super) fn actual_body() -> OwnedNamedPrismBody {
    let fixture = default_resolved_prism_fixture();
    let body = project_named_prism_body(&fixture.publication, &fixture.names).unwrap();
    assert!(body.binds(&fixture.publication, &fixture.names));
    body
}

#[test]
fn actual_resolver_projects_complete_named_geometry_topology_and_tolerances() {
    let body = actual_body();
    assert_eq!(body.names().version(), 2);
    assert_eq!((body.members().len(), body.occurrences().len()), (13, 26));
    assert_eq!((body.faces().len(), body.edge_endpoints().len()), (3, 3));
    assert_eq!((body.coedges().len(), body.wire_traversals().len()), (6, 3));
    assert_eq!(
        body.faces().iter().map(|f| f.trims().len()).sum::<usize>(),
        6
    );
    assert_metadata_roster(&body);
    assert_eq!(body.root().0.len(), 1);
    assert_eq!(body.root().0[0].member.kind, 2);
    assert_member_kinds(&body);
    assert_incidence(&body);
}

fn assert_metadata_roster(body: &OwnedNamedPrismBody) {
    assert_eq!(
        body.members()
            .iter()
            .filter(|m| m.tolerance().is_some())
            .count(),
        8
    );
    assert_eq!(
        body.occurrences()
            .iter()
            .filter(|u| u.local_vertex().is_some())
            .count(),
        12
    );
}

fn assert_member_kinds(body: &OwnedNamedPrismBody) {
    for (kind, expected) in [(2, 1), (3, 1), (4, 3), (5, 3), (6, 3), (7, 2)] {
        assert_eq!(
            body.members()
                .iter()
                .filter(|m| m.key().kind == kind)
                .count(),
            expected
        );
    }
    for member in body.members().iter().filter(|m| m.key().kind == 5) {
        assert!(member.output_slot().is_none());
    }
    assert_geometry_roster(body);
}

fn assert_geometry_roster(body: &OwnedNamedPrismBody) {
    assert_eq!(
        body.members()
            .iter()
            .filter(|m| m.space_curve().is_some())
            .count(),
        3
    );
    assert_eq!(
        body.members()
            .iter()
            .filter(|m| m.rounded_world_point().is_some())
            .count(),
        2
    );
}

fn assert_incidence(body: &OwnedNamedPrismBody) {
    let mut lengths: Vec<_> = body
        .wire_traversals()
        .iter()
        .map(|w| w.coedges().len())
        .collect();
    lengths.sort();
    assert_eq!(lengths, [1, 1, 4]);
    assert_eq!(
        body.edge_endpoints()
            .iter()
            .filter(|e| e.first() == e.last())
            .count(),
        2
    );
    assert_directed_links(body);
}

fn assert_directed_links(body: &OwnedNamedPrismBody) {
    for coedge in body.coedges() {
        assert!(
            body.occurrences()
                .iter()
                .any(|row| row.key() == coedge.occurrence())
        );
    }
    for face in body.faces() {
        for trim in face.trims() {
            assert!(
                body.coedges()
                    .iter()
                    .any(|row| row.occurrence() == trim.occurrence())
            );
        }
    }
}

#[test]
fn real_getters_remain_isolated_after_request_mutation_and_all_inputs_drop() {
    let mut fixture = default_resolved_prism_fixture();
    let body = project_named_prism_body(&fixture.publication, &fixture.names).unwrap();
    let before = getter_snapshot::capture(&body);
    let replay = body.replay_bytes();
    fixture.request.sources[0].world_poles[1][0] = 900.;
    fixture.request.sources[0].revision.0 += 1;
    fixture.request.sources.clear();
    assert_eq!(getter_snapshot::capture(&body), before);
    drop(fixture);
    assert_eq!(getter_snapshot::capture(&body), before);
    assert_eq!(body.replay_bytes(), replay);
}

#[test]
fn projection_copies_curve_storage_and_row_mutation_changes_fresh_receipts_only() {
    let fixture = default_resolved_prism_fixture();
    let mut body = project_named_prism_body(&fixture.publication, &fixture.names).unwrap();
    assert_distinct_curve_storage(&fixture.publication, &body);
    let publication = fixture.publication.replay_bytes().to_vec();
    let getters = getter_snapshot::capture(&body);
    let replay = body.replay_bytes();
    let vertex = body
        .rows
        .occurrences
        .iter_mut()
        .find(|u| u.local_vertex.is_some())
        .unwrap();
    vertex.local_vertex.as_mut().unwrap()[0] += 3.;
    assert_ne!(getter_snapshot::capture(&body), getters);
    assert_ne!(body.replay_bytes(), replay);
    assert_eq!(fixture.publication.replay_bytes(), publication);
}

fn assert_distinct_curve_storage(
    result: &super::super::PublishedNamedPrism,
    body: &OwnedNamedPrismBody,
) {
    let nodes = result.join().projection().nodes();
    for (original, projected) in nodes.iter().zip(body.members()) {
        let Some(original) = &original.curve else {
            continue;
        };
        let projected = projected.space_curve().unwrap();
        if let (MetricCurve::Bezier(a), MetricCurve::Bezier(b)) =
            (original.curve.borrow(), projected.curve)
        {
            assert_eq!(a.poles, b.poles);
            assert_ne!(a.poles.as_ptr(), b.poles.as_ptr());
        }
    }
}

#[test]
fn foreign_actual_operation_map_refuses_without_mutating_reusable_result() {
    let first = default_resolved_prism_fixture();
    let mut request = first.request.clone();
    request.extrusion[2] = 3.;
    let second = resolved_prism_fixture(request);
    let before = first.publication.replay_bytes().to_vec();
    let failure = project_named_prism_body(&first.publication, &second.names)
        .err()
        .unwrap();
    assert_eq!(failure.cause, PrismBodyProjectionCause::PublicationBinding);
    assert_eq!(first.publication.replay_bytes(), before);
    assert!(project_named_prism_body(&first.publication, &first.names).is_ok());
}
