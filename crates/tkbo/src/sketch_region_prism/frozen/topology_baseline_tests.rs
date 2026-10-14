// SPDX-License-Identifier: GPL-2.0-only
//! Owned pre-certificate incidence: one closed cubic extruded along a nonzero vector.
use super::{tests::fresh_projection, *};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::collections::BTreeSet;

#[test]
fn owned_prism_keeps_two_loop_caps_and_one_same_face_seam() {
    let frozen = fresh_projection();
    assert_eq!(frozen.graph.counts, [2, 3, 6, 3, 3, 1, 1]);
    wire_sizes(&frozen);
    assert_eq!(
        frozen
            .graph
            .endpoints
            .iter()
            .filter(|(_, a, b)| a == b)
            .count(),
        2
    );
    assert_eq!(seam_count(&frozen), 1);
    assert!(!frozen.is_valid_body());
}
fn wire_sizes(frozen: &FrozenPrismProjection) {
    let mut sizes: Vec<_> = frozen
        .uses
        .iter()
        .filter(|row| row.key.0.last().unwrap().0.kind == ShapeEnum::Wire as u8)
        .map(|row| row.children.len())
        .collect();
    sizes.sort();
    assert_eq!(sizes, [1, 1, 4]);
}

fn seam_count(frozen: &FrozenPrismProjection) -> usize {
    frozen
        .graph
        .endpoints
        .iter()
        .filter(|(edge, _, _)| {
            let faces: BTreeSet<_> = frozen
                .graph
                .coedges
                .iter()
                .filter(|coedge| coedge.edge == *edge)
                .map(|coedge| coedge.face)
                .collect();
            faces.len() == 1
        })
        .count()
}
#[test]
fn owned_prism_keeps_two_opposite_cumulative_uses_per_edge() {
    let frozen = fresh_projection();
    for (edge, _, _) in &frozen.graph.endpoints {
        let orientations: Vec<_> = frozen
            .uses
            .iter()
            .filter(|row| row.key.0.last().unwrap().0 == *edge)
            .map(|row| row.orientation)
            .collect();
        assert_eq!(orientations.len(), 2);
        assert!(orientations.contains(&Orientation::Forward));
        assert!(orientations.contains(&Orientation::Reversed));
    }
    assert_eq!(
        frozen.coverage.current,
        [crate::sketch_region_validation::StageCoverage::Incomplete; 4]
    );
}

#[test]
fn owned_stored_side_children_are_not_a_cyclic_walk() {
    use FrozenSweepRole::{First, Last};
    let frozen = fresh_projection();
    let wire = frozen
        .uses
        .iter()
        .find(|row| row.children.len() == 4)
        .unwrap();
    let endpoints: Vec<_> = wire
        .children
        .iter()
        .map(|key| local_endpoints(&frozen, key))
        .collect();
    let expected = [(First, Last), (Last, First), (First, First), (Last, Last)];
    assert_eq!(endpoints, expected);
    assert_noncyclic(&endpoints);
}
fn assert_noncyclic(endpoints: &[(FrozenSweepRole, FrozenSweepRole)]) {
    let cyclic =
        (0..endpoints.len()).all(|i| endpoints[i].1 == endpoints[(i + 1) % endpoints.len()].0);
    assert!(
        !cyclic,
        "owned insertion order must remain distinct from traversal evidence"
    );
}

fn local_endpoints(
    frozen: &FrozenPrismProjection,
    key: &FrozenUseKey,
) -> (FrozenSweepRole, FrozenSweepRole) {
    let coedge = frozen
        .graph
        .coedges
        .iter()
        .find(|row| &row.occurrence == key)
        .unwrap();
    let (_, first, last) = frozen
        .graph
        .endpoints
        .iter()
        .find(|row| row.0 == coedge.edge)
        .unwrap();
    directed_roles(*first, *last, coedge.reversed)
}
fn directed_roles(
    first: FrozenPrismKey,
    last: FrozenPrismKey,
    reversed: bool,
) -> (FrozenSweepRole, FrozenSweepRole) {
    if reversed {
        return (last.role, first.role);
    }
    (first.role, last.role)
}
