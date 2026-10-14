// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[test]
fn elevated_constant_cannot_bypass_pre_expansion_admission() {
    let poles = [[3., 4., 5.]; 1025];
    let request = query(&poles);
    let error = certify_bezier_regularity(request, || panic!("refuse before expansion polling"))
        .unwrap_err();
    assert_eq!(error.completion(), BezierRegularityCompletion::Declined);
    assert!(matches!(
        error.cause(),
        BezierRegularityCause::StoredDegree {
            actual: 1024,
            limit: 32
        }
    ));
    assert_eq!(error.bisections(), 0);
    assert_eq!(error.source().poles(), poles);
    assert_eq!(error.source().options(), request.options);
    assert!(
        error
            .to_string()
            .contains("stored degree 1024; expected at most 32")
    );
}
#[test]
fn stored_and_reduced_degree_limits_are_separately_selectable() {
    let poles = [[3., 4., 5.]; 3];
    let mut request = query(&poles);
    request.options.max_stored_degree = 2;
    request.options.roots.max_degree = 0;
    let result = certify_bezier_regularity(request, || false).unwrap();
    assert_eq!(
        result.decision(),
        &BezierRegularityDecision::EverywhereStationary
    );
    assert_eq!(result.denominator_degree(), 0);
    request.options.max_stored_degree = 1;
    assert!(matches!(
        certify_bezier_regularity(request, || false)
            .unwrap_err()
            .cause(),
        BezierRegularityCause::StoredDegree {
            actual: 2,
            limit: 1
        }
    ));
}
#[test]
fn zero_stored_limit_admits_only_degree_zero_sources() {
    let mut request = query(&[[1., 2., 3.]]);
    request.options.max_stored_degree = 0;
    assert_eq!(
        certify_bezier_regularity(request, || false)
            .unwrap()
            .decision(),
        &BezierRegularityDecision::EverywhereStationary
    );
    let configured = BezierRegularityOptions::v1(RootIsolationOptions {
        max_degree: 7,
        ..options(0.1)
    });
    assert_eq!(configured.max_stored_degree, 7);
}
#[test]
fn pre_expansion_admission_preserves_empty_and_mismatched_shape_errors() {
    let poles = [[0.; 3]; 35];
    let mut request = query(&poles);
    request.weights = Some(&[1.]);
    for request in [request, query(&[])] {
        let error =
            certify_bezier_regularity(request, || panic!("shape must fail first")).unwrap_err();
        assert_eq!(error.completion(), BezierRegularityCompletion::Invalid);
        assert!(matches!(
            error.cause(),
            BezierRegularityCause::Coefficients(ResidualError::Shape { .. })
        ));
    }
}
