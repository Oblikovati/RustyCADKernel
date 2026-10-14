// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn generated_dyadic_stationary_parameters_survive_sign_and_reversal() {
    for index in 0..=16 {
        let root = f64::from(index) / 16.;
        check_generated_root(root);
    }
}
fn check_generated_root(root: f64) {
    // Bernstein coefficients of (t-r)^2, all exactly representable for r=k/16.
    let poles = [
        [root * root, 0., 0.],
        [root * root - root, 0., 0.],
        [(1. - root) * (1. - root), 0., 0.],
    ];
    for scale in [-8., 1., 16.] {
        let weights = [scale; 3];
        let mut request = query(&poles);
        request.weights = Some(&weights);
        let result = certify_bezier_regularity(request, || false).unwrap();
        assert_eq!(stationary(&result).len(), 1);
        exact_root(&stationary(&result)[0], root, 2);
    }
    let reversed = [poles[2], poles[1], poles[0]];
    exact_root(&stationary(&report(&reversed))[0], 1. - root, 2);
}
#[test]
fn rational_constant_has_zero_derivative_without_reducing_its_denominator() {
    let poles = [[3., 4., 5.]; 3];
    let mut request = query(&poles);
    request.weights = Some(&[2., -1., 2.]);
    let result = certify_bezier_regularity(request, || false).unwrap();
    assert_eq!(result.denominator_degree(), 2);
    assert_eq!(result.speed_degree(), None);
    assert_eq!(
        result.decision(),
        &BezierRegularityDecision::EverywhereStationary
    );
}
#[test]
fn interval_trimming_retains_native_basis_and_owned_policy() {
    let poles = [[-1., 0., 0.], [1., 0., 0.], [-1., 0., 0.], [1., 0., 0.]];
    for interval in [[0.5, 1.], [0., 0.5], [0.5, 0.5]] {
        let mut request = query(&poles);
        request.interval = interval;
        let result = certify_bezier_regularity(request, || false).unwrap();
        assert_eq!(result.source().interval(), interval);
        exact_root(&stationary(&result)[0], 0.5, 4);
    }
}
