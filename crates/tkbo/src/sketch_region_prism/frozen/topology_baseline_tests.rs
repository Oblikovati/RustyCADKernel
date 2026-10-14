// SPDX-License-Identifier: GPL-2.0-only
//! Owned pre-certificate incidence: one closed cubic extruded along a nonzero vector.
use super::{tests::fresh_projection, *};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use std::collections::BTreeSet;

#[test]
fn owned_prism_keeps_two_loop_caps_and_one_same_face_seam() {
    let frozen = fresh_projection();
    assert_eq!(frozen.graph.counts, [2, 3, 6, 3, 3, 1, 1]);
    let mut sizes: Vec<_> = frozen
        .uses
        .iter()
        .filter(|row| row.key.0.last().unwrap().0.kind == ShapeEnum::Wire as u8)
        .map(|row| row.children.len())
        .collect();
    sizes.sort();
    assert_eq!(sizes, [1, 1, 4]);
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
