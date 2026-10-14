// SPDX-License-Identifier: GPL-2.0-only
//! Independent exact fixtures bind product coverage, correspondence and world construction.
use super::super::RootIsolationOptions;
use super::{operation::*, source::OwnedBezierSource};
fn source(controls: &[[f64; 2]]) -> OwnedBezierSource {
    OwnedBezierSource::capture(controls, None, [0., 1.]).unwrap()
}
fn compute_pair(a: &[[f64; 2]], b: &[[f64; 2]], self_domain: bool) -> PairOperationProof {
    compute(
        [source(a), source(b)],
        self_domain,
        RootIsolationOptions::new(1e-10),
        1e-7,
        &mut || false,
    )
    .unwrap()
}
#[test]
fn real_parabola_pair_certifies_unique_bounded_contact_and_world_error() {
    let a = [[0., 0.], [0.5, 0.], [1., 1.]];
    let b = [[0., 0.5], [0.5, 0.5], [1., -0.5]];
    let proof = compute_pair(&a, &b, false);
    assert_eq!(proof.contacts.len(), 1);
    assert_eq!(proof.contacts[0].u_bounds, [0.5, 0.5]);
    assert_eq!(
        proof.contacts[0].root.bounds().0.compare_f64(0.5).unwrap(),
        std::cmp::Ordering::Equal
    );
    assert_eq!(proof.contacts[0].world.point, [0.5, 0.25]);
    assert_eq!(proof.contacts[0].world.achieved_error, 0.);
    assert_eq!(proof.sources[0].controls, a);
    verify_denominator_certificates(&proof);
    assert_eq!(proof.graph().unwrap().projection.alpha.0, vec![(-2).into()]);
}
#[test]
fn general_self_pipeline_separates_loop_contact_cusp_and_closed_endpoint() {
    let loop_curve = [[3., -9.], [-1., 13.], [-1., -13.], [3., 9.]];
    let cusp = [[3., -3.], [-1., 3.], [-1., -3.], [3., 3.]];
    let closed = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let loop_proof = compute_pair(&loop_curve, &loop_curve, true);
    assert_eq!(loop_proof.contacts.len(), 1);
    assert_eq!(loop_proof.contacts[0].world.point, [0.75, 0.]);
    assert!(compute_pair(&cusp, &cusp, true).contacts.is_empty());
    let closed_proof = compute_pair(&closed, &closed, true);
    assert_eq!(closed_proof.contacts.len(), 1);
    assert_eq!(closed_proof.contacts[0].world.point, [0., 0.]);
    verify_closed_endpoint(&closed_proof);
}
fn verify_closed_endpoint(closed_proof: &PairOperationProof) {
    assert_eq!(
        closed_proof.contacts[0]
            .root
            .bounds()
            .0
            .compare_f64(0.)
            .unwrap(),
        std::cmp::Ordering::Equal
    );
}
#[test]
fn joint_original_equations_filter_extraneous_projection_and_outside_trim() {
    let a = [[0., 0.], [0.5, 0.], [1., 1.]];
    let b = [[0., 2.], [0.5, 2.], [1., 1.]];
    assert_eq!(compute_pair(&a, &b, false).contacts.len(), 1);
    let sources = [
        source(&a),
        OwnedBezierSource::capture(&b, None, [0., 0.5]).unwrap(),
    ];
    let proof = compute(
        sources,
        false,
        RootIsolationOptions::new(1e-9),
        1e-6,
        &mut || false,
    )
    .unwrap();
    assert!(proof.contacts.is_empty());
}

#[test]
fn cancellation_and_invalid_error_do_not_convert_into_completed_empty() {
    let controls = [[0., 0.], [0.5, 0.], [1., 1.]];
    let inputs = [source(&controls), source(&controls)];
    let error = compute(
        inputs.clone(),
        false,
        RootIsolationOptions::new(1e-9),
        1e-6,
        &mut || true,
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("Cancelled"));
    verify_invalid_error(inputs);
    let other = [[0., 0.5], [0.5, 0.5], [1., -0.5]];
    assert_eq!(compute_pair(&controls, &other, false).contacts.len(), 1);
}

fn verify_invalid_error(inputs: [OwnedBezierSource; 2]) {
    let error = compute(
        inputs,
        false,
        RootIsolationOptions::new(1e-9),
        f64::NAN,
        &mut || false,
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("NaN"));
}

fn verify_denominator_certificates(proof: &PairOperationProof) {
    assert!(
        proof
            .source_certificates
            .iter()
            .all(|source| source.denominator.roots().is_empty())
    );
    assert_eq!(
        proof.source_certificates[0].homogeneous.components[2].0,
        vec![1.into()]
    );
}
