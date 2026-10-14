// SPDX-License-Identifier: GPL-2.0-only
//! Literal symbolic rows test projection mechanics only, not geometric incidence acceptance.
use super::{incidence, replay, test_fixture::*};
use crate::sketch_region_prism::{
    frozen::{FrozenSweepRole::*, FrozenUseKey},
    owned_request::names::body_map::ResolvedPrismNames,
};
fn edge_use() -> FrozenUseKey {
    let mut endpoint = cap_endpoint(&names());
    endpoint.0.pop();
    endpoint
}
#[test]
fn closed_rim_retains_both_endpoint_roles_even_with_one_vertex_definition() {
    let names = names();
    let edge = key(2051, First, 6);
    let vertex = key(2052, First, 7);
    let row = incidence::project_endpoints(&names, &edge, &vertex, &vertex).unwrap();
    assert_eq!(row.edge(), names.member(&edge).unwrap());
    assert_eq!(row.first(), names.member(&vertex).unwrap());
    assert_eq!(row.last(), names.member(&vertex).unwrap());
    let mut bytes = Vec::new();
    replay::endpoint(&row, &mut bytes);
    assert_eq!(bytes.len(), 60);
}
#[test]
fn coedge_keeps_full_occurrence_and_explicit_face_wire_edge_and_direction() {
    let names = names();
    let row = incidence::project_coedge(
        &names,
        &edge_use(),
        &key(2056, First, 4),
        &key(2058, First, 5),
        &key(2051, First, 6),
        true,
    )
    .unwrap();
    assert_eq!(row.occurrence(), names.occurrence(&edge_use()).unwrap());
    assert_eq!(row.face(), names.member(&key(2056, First, 4)).unwrap());
    assert_eq!(row.wire(), names.member(&key(2058, First, 5)).unwrap());
    assert_eq!(row.edge(), names.member(&key(2051, First, 6)).unwrap());
    assert!(row.reversed());
    let mut bytes = Vec::new();
    replay::coedge(&row, &mut bytes);
    assert_eq!(bytes.last(), Some(&1));
}
fn side_wire(names: &ResolvedPrismNames) -> (FrozenUseKey, Vec<FrozenUseKey>) {
    let wire = side_wire_key(names);
    let coedges = names
        .use_bindings()
        .iter()
        .filter(|row| row.frozen.0.len() == wire.0.len() + 1 && row.frozen.0.starts_with(&wire.0))
        .map(|row| row.frozen.clone())
        .collect();
    (wire, coedges)
}
#[test]
fn wire_projection_preserves_supplied_cyclic_sequence_without_sorting() {
    let names = names();
    let (wire, mut coedges) = side_wire(&names);
    assert_eq!(coedges.len(), 4);
    let first = incidence::project_wire(&names, &wire, &coedges).unwrap();
    coedges.reverse();
    let reversed = incidence::project_wire(&names, &wire, &coedges).unwrap();
    assert_eq!(first.occurrence(), reversed.occurrence());
    assert_eq!(
        first.coedges().iter().rev().collect::<Vec<_>>(),
        reversed.coedges().iter().collect::<Vec<_>>()
    );
    let mut a = Vec::new();
    let mut b = Vec::new();
    replay::wire(&first, &mut a);
    replay::wire(&reversed, &mut b);
    assert_ne!(a, b);
}

fn side_wire_key(names: &ResolvedPrismNames) -> FrozenUseKey {
    names
        .use_bindings()
        .iter()
        .find(|row| {
            row.frozen
                .0
                .last()
                .is_some_and(|(key, _)| key.kind == 5 && key.role == Generated)
        })
        .unwrap()
        .frozen
        .clone()
}
