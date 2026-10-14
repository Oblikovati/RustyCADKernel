// SPDX-License-Identifier: GPL-2.0-only
use super::source_binding_tests::{bind, rectangle_inputs, report};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::*;
#[test]
fn scalar_controls_weights_native_trim_and_physical_copy_changes_invalidate_proof() {
    let mut inputs = rectangle_inputs();
    let proof = report(&inputs);
    inputs.segments[0].controls[0][0] += 0.5;
    assert!(!bind(&inputs, proof).representation_certified());
    let mut inputs = rectangle_inputs();
    let proof = report(&inputs);
    inputs.segments[0].weights = Some(vec![1., 1.]);
    assert!(!bind(&inputs, proof).representation_certified());
    let mut inputs = rectangle_inputs();
    let proof = report(&inputs);
    inputs.segments[0].original_interval[1] += 0.5;
    assert!(!bind(&inputs, proof).representation_certified());
    let mut inputs = rectangle_inputs();
    let proof = report(&inputs);
    inputs.segments[0].edge = inputs.segments[1].edge.clone();
    assert!(!bind(&inputs, proof).representation_certified());
}
#[test]
fn changed_original_coefficient_or_support_placement_never_matches_old_report() {
    assert_changed_original_rejected();
    let mut inputs = rectangle_inputs();
    let proof = report(&inputs);
    inputs.constructions[0]
        .support
        .as_mut()
        .ok()
        .unwrap()
        .placement
        .origin[0] += 0.5;
    assert!(!bind(&inputs, proof).representation_certified());
}
#[test]
fn signed_zero_basis_and_original_orientation_remain_full_descriptor_bits() {
    let mut inputs = rectangle_inputs();
    let proof = report(&inputs);
    inputs.segments[0].interval[0] = -0.;
    assert!(!bind(&inputs, proof).representation_certified());
    let mut inputs = rectangle_inputs();
    let proof = report(&inputs);
    inputs.originals[0].original.orientation = rustycad_tkg3d::top_abs::Orientation::Reversed;
    assert!(!bind(&inputs, proof).representation_certified());
}
#[test]
fn full_parent_matrix_preserves_reversed_shared_and_located_originals() {
    use rustycad_tkmath::{gp::Transform3, toploc::Location};
    let shape = crate::sketch_region_candidates::tests::rectangle(0., 0., 4., 4.).remove(0);
    let reversed = shape.oriented(rustycad_tkg3d::top_abs::Orientation::Reversed);
    let placed = shape
        .located(&Location::new(Transform3::translation([0., 3., 0.])), false)
        .unwrap();
    let inputs = super::tests::capture(&[shape, reversed, placed]);
    let proof = report(&inputs);
    assert!(bind(&inputs, proof).representation_certified());
    assert_ne!(inputs.originals[0].original, inputs.originals[1].original);
    assert_ne!(
        inputs.originals[0].original.location,
        inputs.originals[2].original.location
    );
}

fn assert_changed_original_rejected() {
    let mut inputs = rectangle_inputs();
    let proof = report(&inputs);
    let CapturedMetricCurve::Line { origin, .. } = &mut inputs.originals[0]
        .representation
        .as_mut()
        .ok()
        .unwrap()
        .curve
    else {
        panic!("line")
    };
    origin[0] += 0.5;
    assert!(!bind(&inputs, proof).representation_certified());
}
