// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn signed_weights_are_admitted_only_with_complete_denominator_proof() {
    let poles = [[0.; 3], [1., 0., 0.], [2., 0., 0.]];
    let mut request = query(&poles);
    request.weights = Some(&[2., -1., 2.]); // D=2-6t+6t² >= 1/2.
    let positive = certify_bezier_regularity(request, || false).unwrap();
    request.weights = Some(&[-2., 1., -2.]);
    let negative = certify_bezier_regularity(request, || false).unwrap();
    assert_eq!(positive.decision(), negative.decision());
    assert_eq!(
        positive.denominator_isolation(),
        negative.denominator_isolation()
    );
    assert_eq!(stationary(&positive).len(), 2);
    assert_eq!(positive.decision(), report(&IRRATIONAL).decision());
    request.weights = Some(&[2., 0., 2.]);
    let endpoints = certify_bezier_regularity(request, || false).unwrap();
    assert_eq!(stationary(&endpoints).len(), 2);
    exact_root(&stationary(&endpoints)[0], 0., 2);
    exact_root(&stationary(&endpoints)[1], 1., 2);
}
#[test]
fn denominator_zeros_are_refused_even_for_removable_factors() {
    let poles = [[3., 4., 5.]; 2];
    for weights in [[1., -1.], [0., 1.], [1., 0.]] {
        let mut request = query(&poles);
        request.weights = Some(&weights);
        let error = certify_bezier_regularity(request, || false).unwrap_err();
        assert_eq!(error.completion(), BezierRegularityCompletion::Declined);
        assert!(matches!(
            error.cause(),
            BezierRegularityCause::DenominatorZeros(_)
        ));
    }
    let mut request = query(&poles);
    request.weights = Some(&[0., 0.]);
    assert!(matches!(
        certify_bezier_regularity(request, || false)
            .unwrap_err()
            .cause(),
        BezierRegularityCause::ZeroDenominator
    ));
}
#[test]
fn reversal_and_rational_reparameterization_preserve_regular_lines() {
    for poles in [[[0.; 3], [2., 0., 0.]], [[2., 0., 0.], [0.; 3]]] {
        for weights in [[1., 2.], [-1., -2.]] {
            let mut request = query(&poles);
            request.weights = Some(&weights);
            let result = certify_bezier_regularity(request, || false).unwrap();
            assert!(matches!(
                result.decision(),
                BezierRegularityDecision::Regular { .. }
            ));
            assert_eq!(result.denominator_degree(), 1);
        }
    }
}
