// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn failure(
    request: BezierRegularityQuery<'_>,
    expected: BezierRegularityCompletion,
) -> BezierRegularityFailure {
    let error = certify_bezier_regularity(request, || false).unwrap_err();
    assert_eq!(error.completion(), expected, "{error}");
    assert_eq!(error.source().options(), request.options);
    assert!(std::error::Error::source(&error).is_some());
    assert!(error.to_string().contains("curve 41"));
    error
}
#[test]
fn root_limits_are_never_widened_and_exhaustion_is_incomplete() {
    let mut request = query(&IRRATIONAL);
    request.options.roots.max_degree = 3;
    let error = failure(request, BezierRegularityCompletion::Declined);
    assert!(matches!(
        error.cause(),
        BezierRegularityCause::Roots {
            stage: BezierRegularityStage::SquaredSpeed,
            cause: RootIsolationError::Degree {
                actual: 4,
                limit: 3
            }
        }
    ));
    for limit in [0, 1, 3] {
        request.options.roots.max_degree = 32;
        request.options.roots.max_bisections = limit;
        let error = failure(request, BezierRegularityCompletion::Incomplete);
        assert_eq!(error.bisections(), limit);
    }
}
#[test]
fn versions_domains_and_root_policies_retain_typed_invalid_causes() {
    let mut request = query(&CUBIC);
    request.options.version = 9;
    assert!(matches!(
        failure(request, BezierRegularityCompletion::Invalid).cause(),
        BezierRegularityCause::Version(9)
    ));
    request.options.version = 1;
    request.options.roots.version = 9;
    failure(request, BezierRegularityCompletion::Invalid);
    request.options.roots = options(1e-6);
    for interval in [[-1., 1.], [0., 2.], [1., 0.], [0., f64::INFINITY]] {
        request.interval = interval;
        assert!(matches!(
            failure(request, BezierRegularityCompletion::Invalid).cause(),
            BezierRegularityCause::Domain(_)
        ));
    }
    request.interval = [0., 1.];
    request.options.roots = options(0.);
    failure(request, BezierRegularityCompletion::Invalid);
}
#[test]
fn malformed_coefficients_and_placements_are_invalid() {
    failure(query(&[]), BezierRegularityCompletion::Invalid);
    let mut request = query(&CUBIC);
    request.weights = Some(&[1.]);
    failure(request, BezierRegularityCompletion::Invalid);
    request.weights = Some(&[1., 1., f64::NAN, 1.]);
    failure(request, BezierRegularityCompletion::Invalid);
    request.weights = None;
    request.placement.linear[0][0] = f64::INFINITY;
    failure(request, BezierRegularityCompletion::Invalid);
    request.placement.linear[0][0] = 1.;
    request.placement.origin[1] = f64::NAN;
    failure(request, BezierRegularityCompletion::Invalid);
    failure(
        query(&[[f64::INFINITY, 0., 0.]]),
        BezierRegularityCompletion::Invalid,
    );
}
fn counted(request: BezierRegularityQuery<'_>) -> (BezierRegularityReport, usize) {
    let mut polls = 0;
    let report = certify_bezier_regularity(request, || {
        polls += 1;
        false
    })
    .unwrap();
    (report, polls)
}
fn cancel_at(request: BezierRegularityQuery<'_>, stop: usize) -> BezierRegularityFailure {
    let mut current = 0;
    let error = certify_bezier_regularity(request, || {
        current += 1;
        current == stop
    })
    .unwrap_err();
    assert_eq!(error.completion(), BezierRegularityCompletion::Cancelled);
    error
}
#[test]
fn cancellation_at_every_short_operation_poll_never_publishes_partial_success() {
    let poles = [[0.; 3], [1., 2., 3.]];
    let request = query(&poles);
    let (expected, polls) = counted(request);
    for stop in 1..=polls {
        assert_eq!(cancel_at(request, stop).source(), expected.source());
    }
    assert_eq!(
        certify_bezier_regularity(request, || false).unwrap(),
        expected
    );
}
#[test]
fn cancellation_retains_completed_subdivisions_and_inputs() {
    let request = query(&IRRATIONAL);
    let (expected, polls) = counted(request);
    for stop in [1, polls / 2, polls] {
        let error = cancel_at(request, stop);
        assert_eq!(error.source(), expected.source());
        assert!(error.bisections() <= expected.bisections());
        if stop == polls {
            assert_eq!(error.bisections(), expected.bisections());
        }
    }
}
#[test]
fn denominator_degree_limit_is_a_typed_refusal_before_speed_construction() {
    let mut request = query(&CUBIC);
    request.weights = Some(&[1., 2., 3., 5.]);
    request.options.roots.max_degree = 2;
    let error = failure(request, BezierRegularityCompletion::Declined);
    assert!(matches!(
        error.cause(),
        BezierRegularityCause::Roots {
            stage: BezierRegularityStage::Denominator,
            cause: RootIsolationError::Degree {
                actual: 3,
                limit: 2
            }
        }
    ));
}
