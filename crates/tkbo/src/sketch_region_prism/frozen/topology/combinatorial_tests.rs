// SPDX-License-Identifier: GPL-2.0-only
use super::super::traversal_tests::ordered_projection as fresh_projection;
use super::{fixtures::NamedCellComplex, tests::*, *};
use rustycad_tkg3d::top_abs::Orientation;

#[test]
fn independent_tetrahedron_and_two_single_coedge_disks_certify_sphere_incidence() {
    let tetrahedron = NamedCellComplex::tetrahedron().projection();
    assert_eq!(certified(&tetrahedron).counts, [4, 6, 12, 4, 4, 1, 1]);
    let disks = NamedCellComplex {
        edges: vec![(0, 0)],
        faces: vec![vec![1], vec![-1]],
    }
    .projection();
    assert_eq!(certified(&disks).counts, [1, 1, 2, 2, 2, 1, 1]);
}
#[test]
fn wraparound_is_required_even_when_all_linear_neighbors_connect() {
    let open = NamedCellComplex {
        edges: vec![(0, 1), (1, 2)],
        faces: vec![vec![1, 2]],
    }
    .projection();
    rejected(&open, TopologyInvariant::CyclicWires);
    let one = NamedCellComplex {
        edges: vec![(0, 1)],
        faces: vec![vec![1]],
    }
    .projection();
    rejected(&one, TopologyInvariant::CyclicWires);
}
#[test]
fn one_and_three_directed_uses_cannot_replace_exact_edge_pairing() {
    for faces in [vec![vec![1]], vec![vec![1], vec![-1], vec![1]]] {
        let source = NamedCellComplex {
            edges: vec![(0, 0)],
            faces,
        }
        .projection();
        let failure = rejected(&source, TopologyInvariant::OppositeEdgeUses);
        assert!(matches!(
            failure.witness.actual,
            TopologyActual::Count(1 | 3)
        ));
    }
}
#[test]
fn equal_orientations_fail_even_with_two_closed_uses_and_sphere_counts() {
    let source = NamedCellComplex {
        edges: vec![(0, 0)],
        faces: vec![vec![1], vec![1]],
    }
    .projection();
    let failure = rejected(&source, TopologyInvariant::OppositeEdgeUses);
    assert!(matches!(
        failure.witness.actual,
        TopologyActual::Direction(Orientation::Forward)
    ));
}
#[test]
fn pinched_vertex_has_two_link_cycles_despite_two_opposite_uses_per_edge() {
    let source = NamedCellComplex::double_tetrahedron(true).projection();
    let failure = rejected(&source, TopologyInvariant::CyclicVertexLinks);
    assert_eq!(failure.witness.entities.len(), 1);
}
#[test]
fn disconnected_shell_faces_fail_after_each_local_vertex_link_passes() {
    let source = NamedCellComplex::double_tetrahedron(false).projection();
    let failure = rejected(&source, TopologyInvariant::ConnectedFaces);
    assert!(matches!(failure.witness.actual, TopologyActual::Count(4)));
}
#[test]
fn torus_word_passes_local_manifold_checks_but_refuses_sphere_euler_domain() {
    let source = NamedCellComplex {
        edges: vec![(0, 0), (0, 0)],
        faces: vec![vec![1, 2, -1, -2]],
    }
    .projection();
    let failure = rejected(&source, TopologyInvariant::SphereEuler);
    assert!(matches!(failure.witness.actual, TopologyActual::Euler(0)));
}
#[test]
fn whole_shell_reversal_preserves_combinatorial_orientation_without_outward_claim() {
    let mut source = fresh_projection();
    let original = certified(&source);
    for occurrence in &mut source.uses {
        if occurrence.key.0.len() >= 2 {
            occurrence.orientation = reverse(occurrence.orientation);
        }
    }
    let reversed = certified(&source);
    assert_eq!(original.counts, reversed.counts);
    assert_ne!(original.replay_bytes(), reversed.replay_bytes());
}
#[test]
fn reversing_one_face_and_descendants_breaks_provided_orientation_pairing() {
    let mut source = fresh_projection();
    let face = source
        .uses
        .iter()
        .find(|row| row.key.0.len() == 3)
        .unwrap()
        .key
        .clone();
    for occurrence in &mut source.uses {
        if occurrence.key.0.starts_with(&face.0) {
            occurrence.orientation = reverse(occurrence.orientation);
        }
    }
    rejected(&source, TopologyInvariant::OppositeEdgeUses);
}
pub(super) fn reverse(orientation: Orientation) -> Orientation {
    match orientation {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
        _ => panic!("fixture orientation {orientation:?}; expected Forward/Reversed"),
    }
}

#[test]
fn all_sixteen_tetrahedron_face_flips_have_exactly_two_consistent_orientations() {
    let fixture = NamedCellComplex::tetrahedron().projection();
    for mask in 0u64..16 {
        let mut source = fixture.clone();
        flip_tetrahedron_faces(&mut source, mask);
        if mask == 0 || mask == 15 {
            certified(&source);
        } else {
            rejected(&source, TopologyInvariant::OppositeEdgeUses);
        }
    }
}
fn flip_tetrahedron_faces(source: &mut super::super::FrozenPrismProjection, mask: u64) {
    for occurrence in &mut source.uses {
        let Some((face, _)) = occurrence.key.0.get(2) else {
            continue;
        };
        if mask & (1 << (face.source.0 - 900)) != 0 {
            occurrence.orientation = reverse(occurrence.orientation);
        }
    }
}
