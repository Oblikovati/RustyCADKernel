// SPDX-License-Identifier: GPL-2.0-only
//! Inspect actual stored representations and geometry, never infer isolation from Clone.
use super::{copy_tests::actual_candidate_copy, fixtures::*};
use crate::sketch_region::*;
use crate::sketch_region_candidates::{CandidatePhase, candidate_representation_count};
use crate::sketch_region_source_conformance::{capture_originals, same_space_representation};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};

#[derive(Debug, PartialEq, Eq)]
pub(super) struct CircleInputStamp {
    representations: usize,
    geometry_bits: Vec<u64>,
    range_bits: [u64; 2],
}
pub(super) fn circle_input_stamp(shape: &Shape) -> CircleInputStamp {
    use rustycad_tkg3d::geom::GeomCircle;
    let (curve, first, last) = BRepTool::curve(&edge(shape).unwrap()).unwrap();
    let circle = curve.cast::<GeomCircle>();
    let geometry_bits = circle_geometry_bits(circle.get().unwrap());
    CircleInputStamp {
        representations: candidate_representation_count(shape),
        geometry_bits,
        range_bits: [first.to_bits(), last.to_bits()],
    }
}
fn circle_geometry_bits(circle: &rustycad_tkg3d::geom::GeomCircle) -> Vec<u64> {
    use rustycad_tkg3d::geom::GeomConic;
    let position = circle.position();
    let point = position.location();
    let mut bits = vec![
        circle.radius().to_bits(),
        point.x().to_bits(),
        point.y().to_bits(),
        point.z().to_bits(),
    ];
    append_circle_directions(&position, &mut bits);
    bits
}
fn append_circle_directions(position: &rustycad_tkmath::gp::Axis2, bits: &mut Vec<u64>) {
    for direction in [
        position.direction(),
        position.x_direction(),
        position.y_direction(),
    ] {
        bits.extend([
            direction.x().to_bits(),
            direction.y().to_bits(),
            direction.z().to_bits(),
        ]);
    }
}

#[test]
fn source_and_arrangement_storage_are_unchanged_after_copy_success_and_cancellation() {
    let edges = crate::sketch_region_candidates::tests::rectangle(0., 0., 2., 2.);
    let before = capture_originals(&binding_sources(&edges)).unwrap();
    let counts: Vec<_> = edges.iter().map(candidate_representation_count).collect();
    let owned = crate::sketch_region_candidates::tests::arranged(&edges);
    assert_copy_storage(&owned, None);
    assert_copy_storage(&owned, Some(CandidatePhase::Generation));
    let after = capture_originals(&binding_sources(&edges)).unwrap();
    assert_original_descriptors(&before, &after);
    assert_eq!(
        counts,
        edges
            .iter()
            .map(candidate_representation_count)
            .collect::<Vec<_>>()
    );
}
fn assert_copy_storage(owned: &OwnedArrangement, stop: Option<CandidatePhase>) {
    let input_edges: Vec<_> = owned
        .relations
        .iter()
        .filter(|row| row.output.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Edge)
        .map(|row| &row.output)
        .collect();
    let before: Vec<_> = input_edges
        .iter()
        .map(|shape| candidate_representation_count(shape))
        .collect();
    let _copied = actual_candidate_copy(owned, stop);
    let after: Vec<_> = input_edges
        .iter()
        .map(|shape| candidate_representation_count(shape))
        .collect();
    assert_eq!(before, after);
}

fn assert_original_descriptors(
    before: &[crate::sketch_region_source_conformance::OriginalCurveCapture],
    after: &[crate::sketch_region_source_conformance::OriginalCurveCapture],
) {
    assert_eq!(before.len(), after.len());
    for (first, last) in before.iter().zip(after) {
        assert_eq!(first.original, last.original);
        assert!(same_space_representation(
            first.representation.as_ref().ok().unwrap(),
            last.representation.as_ref().ok().unwrap()
        ));
    }
}
