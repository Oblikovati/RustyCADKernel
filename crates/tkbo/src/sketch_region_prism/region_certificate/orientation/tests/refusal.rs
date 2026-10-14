// SPDX-License-Identifier: GPL-2.0-only
//! Isolated sign-rule refusals are distinct from full-entry stale-binding refusals.
use super::*;
#[test]
fn isolated_facing_and_traversal_cases_have_independent_native_signs() {
    use Orientation::{Forward as F, Reversed as R};
    let mut binding = fresh_binding();
    for (face, reverse, native) in [(F, false, 1), (F, true, -1), (R, false, -1), (R, true, 1)] {
        binding.region.source_candidate.face.orientation = face;
        binding.region.boundary_reversed = reverse;
        assert_eq!(
            signs::native_winding(&binding.region)
                .ok()
                .unwrap()
                .factors()
                .3,
            native
        );
    }
}
#[test]
fn isolated_zero_jacobian_and_nonjordan_winding_are_typed_refusals() {
    let mut binding = fresh_binding();
    binding.region.selected_winding = -1;
    assert!(matches!(
        signs::native_winding(&binding.region),
        Err(PrismOrientationEvaluationCause::Winding {
            actual: -1,
            expected: 1
        })
    ));
}
#[test]
fn isolated_zero_jacobian_refuses_orientation_only() {
    let binding = fresh_binding();
    let zero = affine_prism_jacobian(
        binding.region.copied.support.frame,
        binding.region.copied.support.placement,
        [0.; 3],
        || false,
    )
    .unwrap();
    assert!(matches!(
        signs::determinant_sign(&zero),
        Err(PrismOrientationEvaluationCause::ZeroJacobian)
    ));
}
fn fresh_binding() -> CurrentPrismBinding {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    capture_current_prism_binding(&bridge, &binding_sources(&shapes), &receipt, &mut || false)
        .ok()
        .unwrap()
}
#[test]
fn isolated_flipped_faces_preserve_exact_role_and_expected_orientation() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let evidence = oriented(&bridge, &shapes, &receipt);
    for row in evidence.faces() {
        let mut domain = row.domain().clone();
        domain.face.orientation = rustycad_tkg3d::top_abs::reverse(domain.face.orientation);
        let cause = faces::inspect_face(&domain, 1, 1).err().unwrap();
        assert!(
            matches!(cause, PrismOrientationEvaluationCause::Face { domain: rejected, actual, expected }
            if rejected.face == domain.face && actual != expected && expected == row.expected)
        );
    }
}
#[test]
fn isolated_each_trim_flip_is_attributed_including_both_seam_uses() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let evidence = oriented(&bridge, &shapes, &receipt);
    for row in evidence.faces() {
        for index in 0..row.trims().len() {
            let mut domain = row.domain().clone();
            domain.trims[index].output.orientation =
                rustycad_tkg3d::top_abs::reverse(domain.trims[index].output.orientation);
            let cause = faces::inspect_face(&domain, 1, 1).err().unwrap();
            assert!(
                matches!(cause,PrismOrientationEvaluationCause::Trim { authored, actual, expected }
                if *authored == domain.trims[index] && actual != expected)
            );
        }
    }
}
#[test]
fn internal_and_external_orientations_never_become_signed_faces() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let evidence = oriented(&bridge, &shapes, &receipt);
    for unsupported in [Orientation::Internal, Orientation::External] {
        let mut domain = evidence.faces()[0].domain().clone();
        domain.face.orientation = unsupported;
        assert!(matches!(faces::inspect_face(&domain,1,1),
            Err(PrismOrientationEvaluationCause::Unsupported { entity, actual })
            if *entity == domain.face && actual == unsupported));
    }
}
#[test]
fn explicit_behavior_and_embedding_failures_remain_distinct() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut policy = options();
    policy.version = 0;
    let error = attempt(&bridge, &shapes, &receipt, policy, || false)
        .err()
        .unwrap();
    assert!(matches!(
        error.cause,
        PrismOrientationCause::Version {
            actual: 0,
            expected: 1
        }
    ));
    assert_eq!(error.requested, policy);
}
#[test]
fn embedding_policy_failure_preserves_original_typed_cause() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut policy = options();
    policy.embedding.regularity.max_stored_degree = 0;
    let error = attempt(&bridge, &shapes, &receipt, policy, || false)
        .err()
        .unwrap();
    assert!(matches!(error.cause, PrismOrientationCause::Embedding(_)));
    assert_eq!(error.phase, PrismOrientationPhase::Embedding);
}

#[test]
fn exact_determinant_sign_survives_oblique_reflected_and_underflow_domains() {
    let binding = fresh_binding();
    let mut frame = binding.region.copied.support.frame;
    let placement = binding.region.copied.support.placement;
    for (columns, extrusion, expected) in [
        ([[2., 1.], [0., 3.], [0., 0.]], [1., 2., 4.], 1),
        ([[2., 1.], [0., -3.], [0., 0.]], [1., 2., 4.], -1),
        (
            [[1., 0.], [0., 1.], [0., 0.]],
            [0., 0., f64::from_bits(1)],
            1,
        ),
    ] {
        frame.linear = columns;
        let exact = affine_prism_jacobian(frame, placement, extrusion, || false).unwrap();
        assert_eq!(signs::determinant_sign(&exact).ok().unwrap(), expected);
    }
}
