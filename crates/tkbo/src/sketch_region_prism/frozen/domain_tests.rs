// SPDX-License-Identifier: GPL-2.0-only
//! Independent bounded-domain checks allocate at most eighteen small poles.
use super::{domain::*, *};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
#[test]
fn occurrence_bounds_refuse_overflow_without_allocating() {
    assert!(check_count(0).is_ok());
    assert!(check_count(MAXIMUM_USES).is_ok());
    for actual in [MAXIMUM_USES + 1, usize::MAX] {
        assert!(
            matches!(check_count(actual), Err(PrismFreezeCause::LimitExceeded {
            actual: count, maximum: MAXIMUM_USES, ..
        }) if count == actual)
        );
    }
}
fn bezier(poles: usize, weights: Option<Vec<f64>>) -> CapturedMetricCurve<3> {
    CapturedMetricCurve::Bezier {
        poles: vec![[0.; 3]; poles],
        weights,
    }
}
#[test]
fn polynomial_bounds_include_constants_and_reject_excess_without_truncation() {
    assert!(polynomial(&bezier(1, None)).is_ok());
    assert!(polynomial(&bezier(MAXIMUM_POLES, None)).is_ok());
    for poles in [0, MAXIMUM_POLES + 1] {
        assert!(matches!(polynomial(&bezier(poles, None)),
            Err(PrismFreezeCause::PolynomialPoles { actual, .. }) if actual == poles));
    }
    assert!(matches!(
        polynomial(&bezier(2, Some(vec![1.; 2]))),
        Err(PrismFreezeCause::RationalWeights { actual: 2 })
    ));
}
#[test]
fn fixed_policy_records_exact_endpoint_admission_and_refusal_limits() {
    let projection = tests::fresh_projection();
    let policy = projection.capture_policy();
    assert_eq!(policy.maximum_directed_occurrences, MAXIMUM_USES);
    assert_eq!(policy.maximum_polynomial_poles, MAXIMUM_POLES);
    assert_eq!(policy.exact_endpoint_spatial_limit, 0.);
}
