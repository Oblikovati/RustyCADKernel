// SPDX-License-Identifier: GPL-2.0-only
//! The irrational joint root has an independently bounded construction error, not a residual.
use super::super::{RootIsolationOptions, certificate_arithmetic::ExactRatio};
use super::{operation::compute, source::OwnedBezierSource};
#[test]
fn irrational_curved_pair_retains_joint_root_and_certified_world_radius() {
    let proof = irrational_proof();
    assert_eq!(proof.contacts.len(), 1);
    let contact = &proof.contacts[0];
    assert!(contact.root.width().compare_f64(0.).unwrap().is_gt());
    assert_eq!(
        proof
            .graph()
            .unwrap()
            .candidate_certificate
            .polynomial
            .primitive(2)
            .unwrap()
            .0,
        vec![(-1).into(), 0.into(), 2.into()]
    );
    verify_exact_sqrt_half_bracket(&contact.root);
    verify_independent_world_error(contact, 1e-6);
}
pub(super) fn irrational_proof() -> super::operation::PairOperationProof {
    let source =
        |controls: &[[f64; 2]]| OwnedBezierSource::capture(controls, None, [0., 1.]).unwrap();
    let sources = [
        source(&[[0., 0.], [0.5, 0.], [1., 1.]]),
        source(&[[0., 1.], [0.5, 1.], [1., 0.]]),
    ];
    compute(
        sources,
        false,
        RootIsolationOptions::new(1e-9),
        1e-6,
        &mut || false,
    )
    .unwrap()
}
pub(super) fn verify_exact_sqrt_half_bracket(root: &super::super::RootBracket) {
    let lower = ExactRatio::from_dyadic(root.bounds().0);
    let upper = ExactRatio::from_dyadic(root.bounds().1);
    let half = ExactRatio::from_finite(0.5);
    assert!(lower.multiply(&lower) < half);
    assert!(upper.multiply(&upper) > half);
}
pub(super) fn verify_independent_world_error(
    contact: &super::operation::PairContact,
    requested: f64,
) {
    let x = ExactRatio::from_finite(contact.world.point[0]);
    let y = ExactRatio::from_finite(contact.world.point[1]).subtract(&ExactRatio::from_finite(0.5));
    let lower = x.subtract(&ExactRatio::from_dyadic(contact.root.bounds().0));
    let upper = x.subtract(&ExactRatio::from_dyadic(contact.root.bounds().1));
    let x_squared = lower.multiply(&lower).max(upper.multiply(&upper));
    let bound_squared = x_squared.add(&y.multiply(&y));
    let radius = ExactRatio::from_finite(contact.world.achieved_error);
    assert!(bound_squared <= radius.multiply(&radius));
    assert!(contact.world.achieved_error > 0. && contact.world.achieved_error <= requested);
    assert!(contact.world.bounds[1][0] <= 0.5 && contact.world.bounds[1][1] >= 0.5);
}
#[test]
fn permuted_rational_pair_certifies_exact_third_root_and_actual_world_error() {
    let proof = permuted_rational_proof();
    assert_eq!(
        proof
            .graph()
            .unwrap()
            .candidate_certificate
            .polynomial
            .primitive(1)
            .unwrap()
            .0,
        vec![(-1).into(), 3.into()]
    );
    let contact = &proof.contacts[0];
    let third = ExactRatio::new(1.into(), 3.into());
    assert!(ExactRatio::from_dyadic(contact.root.bounds().0) <= third);
    assert!(ExactRatio::from_dyadic(contact.root.bounds().1) >= third);
    assert!(contact.u_bounds[0] <= 0.5 && contact.u_bounds[1] >= 0.5);
    assert_exact_third_world_radius(contact);
}
fn permuted_rational_proof() -> super::operation::PairOperationProof {
    let left = OwnedBezierSource::capture(&[[0., 1.], [1., 0.]], None, [0., 1.]).unwrap();
    let right =
        OwnedBezierSource::capture(&[[0., 0.], [0.5, 1.]], Some(&[1., 2.]), [0., 1.]).unwrap();
    compute(
        [left, right],
        false,
        RootIsolationOptions::new(0.01),
        5e-17,
        &mut || false,
    )
    .unwrap()
}
fn assert_exact_third_world_radius(contact: &super::operation::PairContact) {
    let exact = [
        ExactRatio::new(1.into(), 3.into()),
        ExactRatio::new(2.into(), 3.into()),
    ];
    let mut squared = ExactRatio::from_finite(0.);
    for (coordinate, expected) in contact.world.point.iter().zip(exact) {
        let delta = ExactRatio::from_finite(*coordinate).subtract(&expected);
        squared = squared.add(&delta.multiply(&delta));
    }
    let radius = ExactRatio::from_finite(contact.world.achieved_error);
    assert!(squared <= radius.multiply(&radius));
    assert!(contact.world.achieved_error <= 5e-17);
}
