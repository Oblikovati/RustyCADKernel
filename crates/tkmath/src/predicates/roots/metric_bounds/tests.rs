// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::predicates::roots::{RootIsolationError, RootIsolationOptions};
pub(super) fn identity_embedding() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
pub(super) fn default_plane_embedding() -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
pub(super) fn bezier<const DIM: usize>(poles: &[[f64; DIM]]) -> MetricCurve<'_, DIM> {
    MetricCurve::Bezier(MetricBezier {
        poles,
        weights: None,
    })
}
pub(super) fn query<'a>(
    curve: &'a [[f64; 3]],
    pcurve: &'a [[f64; 2]],
    limit: f64,
) -> PlanarResidualQuery<'a> {
    PlanarResidualQuery {
        curve: bezier(curve),
        curve_embedding: identity_embedding(),
        pcurve: bezier(pcurve),
        plane_embedding: default_plane_embedding(),
        plane_placement: identity_embedding(),
        interval: [0., 1.],
        spatial_limit: limit,
        options: RootIsolationOptions::new(0.001),
    }
}
pub(super) fn weighted_query<'a>(
    curve: &'a [[f64; 3]],
    pcurve: &'a [[f64; 2]],
    weights: &'a [f64],
    limit: f64,
) -> PlanarResidualQuery<'a> {
    let mut request = query(curve, pcurve, limit);
    request.curve = MetricCurve::Bezier(MetricBezier {
        poles: curve,
        weights: Some(weights),
    });
    request.pcurve = MetricCurve::Bezier(MetricBezier {
        poles: pcurve,
        weights: Some(weights),
    });
    request
}
#[test]
fn exact_closed_cubic_has_zero_residual_over_its_entire_domain() {
    let curve = [[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 0.]];
    let planar = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let result = certify_planar_residual(query(&curve, &planar, 0.), || false).unwrap();
    let ResidualDecision::Within {
        proven_squared_upper,
    } = result
    else {
        panic!("expected complete zero residual");
    };
    assert_eq!(
        proven_squared_upper.compare_f64(0.).unwrap(),
        std::cmp::Ordering::Equal
    );
}
#[test]
fn interior_peak_is_detected_without_endpoint_or_midpoint_sampling() {
    let curve = [[0., 0., 0.], [0., 0., 4.], [0., 0., 0.]];
    let planar = [[0., 0.], [0., 0.]];
    let result = certify_planar_residual(query(&curve, &planar, 1.99), || false).unwrap();
    assert!(matches!(
        result,
        ResidualDecision::Exceeds {
            witness: ResidualViolation::Stationary(_)
        }
    ));
    assert!(matches!(
        certify_planar_residual(query(&curve, &planar, 2.), || false).unwrap(),
        ResidualDecision::Within { .. }
    ));
}

#[test]
fn nonunit_rational_weights_preserve_amplitude_and_constant_offsets() {
    let curve = [[0., 0., 0.5], [1., 0., 0.5], [2., 0., 0.5]];
    let planar = [[0., 0.], [1., 0.], [2., 0.]];
    let weights = [2., 7., 3.];
    assert!(matches!(
        certify_planar_residual(weighted_query(&curve, &planar, &weights, 0.5), || false).unwrap(),
        ResidualDecision::Within { .. }
    ));
    assert!(matches!(
        certify_planar_residual(weighted_query(&curve, &planar, &weights, 0.49), || false).unwrap(),
        ResidualDecision::Exceeds { .. }
    ));
}
#[test]
fn trimmed_domain_excludes_actual_interior_peak_without_changing_curve() {
    let curve = [[0., 0., 0.], [0., 0., 4.], [0., 0., 0.]];
    let planar = [[0., 0.], [0., 0.]];
    let mut request = query(&curve, &planar, 1.);
    request.interval = [0., 0.125];
    assert!(matches!(
        certify_planar_residual(request, || false).unwrap(),
        ResidualDecision::Within { .. }
    ));
    assert_eq!(curve[1][2], 4.);
}
#[test]
fn cancelled_and_degree_exhausted_requests_never_return_a_certificate() {
    let curve = [[0., 0., 0.], [0., 0., 4.], [0., 0., 0.]];
    let planar = [[0., 0.], [0., 0.]];
    assert!(matches!(
        certify_planar_residual(query(&curve, &planar, 1.), || true),
        Err(ResidualError::Roots(
            super::super::RootIsolationError::Cancelled
        ))
    ));
    let mut request = query(&curve, &planar, 1.);
    request.options.max_degree = 1;
    assert!(matches!(
        certify_planar_residual(request, || false),
        Err(ResidualError::Roots(
            super::super::RootIsolationError::Degree { .. }
        ))
    ));
}
#[test]
fn actual_line_parameters_and_raw_scaled_placement_avoid_endpoint_rounding() {
    let curve = [[0., 0., 0.], [1., 0., 0.]];
    let planar = [[0., 0.], [1., 0.]];
    let request = placed_line_query(&curve, &planar);
    assert!(matches!(
        certify_planar_residual(request, || false).unwrap(),
        ResidualDecision::Within { .. }
    ));
}

fn independent_peak_exceeds_samples(limit: f64) {
    use num_bigint::BigInt;
    let scale = BigInt::from(1) << 1074usize;
    let threshold = super::coefficients::scalar(limit, "reference limit", 0).unwrap();
    let sample_denominator = BigInt::from(22).pow(23);
    for index in 0..=22 {
        let sample = BigInt::from(23 * index) * BigInt::from(22 - index).pow(22);
        assert!(sample * &scale <= &threshold * &sample_denominator);
    }
    assert!(BigInt::from(22).pow(22) * scale > threshold * BigInt::from(23).pow(22));
}
#[test]
fn independent_stationary_peak_between_twenty_three_samples_is_rejected() {
    let limit = 0.3758881921123227;
    independent_peak_exceeds_samples(limit);
    let mut curve = [[0.; 3]; 24];
    curve[1][2] = 1.;
    let planar = [[0.; 2]; 2];
    let mut request = query(&curve, &planar, limit);
    request.options.max_degree = 64;
    let result = certify_planar_residual(request, || false).unwrap();
    assert!(matches!(
        result,
        ResidualDecision::Exceeds {
            witness: ResidualViolation::Stationary(_)
        }
    ));
}
#[test]
fn subnormal_and_huge_residual_squares_do_not_underflow_or_overflow() {
    for height in [f64::from_bits(1), 1e308] {
        check_constant_height(height);
    }
}
fn check_constant_height(height: f64) {
    let planar = [[0.; 2]; 2];
    let curve = [[0., 0., height], [0., 0., height]];
    let result = certify_planar_residual(query(&curve, &planar, height), || false).unwrap();
    let ResidualDecision::Within {
        proven_squared_upper,
    } = result
    else {
        panic!("expected exact constant bound");
    };
    assert_eq!(
        proven_squared_upper.compare_f64(0.).unwrap(),
        std::cmp::Ordering::Greater
    );
    assert!(matches!(
        certify_planar_residual(query(&curve, &planar, 0.), || false).unwrap(),
        ResidualDecision::Exceeds { .. }
    ));
}
#[test]
fn mixed_nonpositive_weights_and_nonfinite_inputs_are_not_certified() {
    let curve = [[0.; 3]; 2];
    let planar = [[0.; 2]; 2];
    let weights = [1., -1.];
    let mut request = query(&curve, &planar, 0.);
    request.curve = MetricCurve::Bezier(MetricBezier {
        poles: &curve,
        weights: Some(&weights),
    });
    assert!(matches!(
        certify_planar_residual(request, || false),
        Err(ResidualError::Value {
            role: "weight",
            index: 1,
            value: -1.
        })
    ));
}
#[test]
fn nonfinite_limit_keeps_actual_value_and_expected_shape() {
    let curve = [[0.; 3]; 2];
    let planar = [[0.; 2]; 2];
    let request = query(&curve, &planar, f64::NAN);
    let error = certify_planar_residual(request, || false).unwrap_err();
    assert!(error.to_string().contains("spatial_limit[0]=NaN"));
    assert!(error.to_string().contains("expected"));
}
#[test]
fn actual_subdivision_budget_exhaustion_keeps_coverage_incomplete() {
    let curve = [[0., 0., 0.], [0., 0., 4.], [0., 0., 0.]];
    let planar = [[0.; 2]; 2];
    let mut request = query(&curve, &planar, 1.99);
    request.options.max_bisections = 0;
    request.options.max_width = 1e-12;
    assert!(matches!(
        certify_planar_residual(request, || false),
        Err(ResidualError::Roots(
            super::super::RootIsolationError::SubdivisionLimit { limit: 0 }
        ))
    ));
}

fn placed_line_query<'a>(curve: &'a [[f64; 3]], planar: &'a [[f64; 2]]) -> PlanarResidualQuery<'a> {
    let mut request = query(curve, planar, 0.);
    request.curve = MetricCurve::Line {
        origin: [0.; 3],
        direction: [1., 0., 0.],
    };
    request.pcurve = MetricCurve::Line {
        origin: [0.; 2],
        direction: [1., 0.],
    };
    request.curve_embedding.scale = 3.;
    request.plane_placement.scale = 3.;
    request.curve_embedding.origin = [7., 11., -13.];
    request.plane_placement.origin = [7., 11., -13.];
    request.interval = [-10., 20.];
    request
}

#[test]
fn mismatched_control_arrays_preserve_offending_shape_and_expected_contract() {
    let curve = [[0.; 3]; 2];
    let planar = [[0.; 2]; 2];
    let weights = [1.];
    let mut request = query(&curve, &planar, 0.);
    request.curve = MetricCurve::Bezier(MetricBezier {
        poles: &curve,
        weights: Some(&weights),
    });
    let cause = certify_planar_residual(request, || false).unwrap_err();
    assert!(matches!(
        cause,
        ResidualError::Shape {
            representation: "curve",
            poles: 2,
            weights: Some(1)
        }
    ));
    assert_control_array_diagnostic(&cause);
}

#[path = "../../../../tests/support/metric_residual_cases.rs"]
mod accepted_fixture_workloads;
use crate::predicates::roots as metric_api;
#[test]
fn accepted_performance_cases_match_functional_metric_decisions() {
    let workloads = accepted_fixture_workloads::accepted_metric_workloads();
    assert_eq!(workloads.len(), 6);
    for workload in workloads {
        assert!(!workload.name.is_empty());
        workload.verify();
    }
}

#[test]
fn exact_singleton_checks_actual_mapped_point_error_against_original_source() {
    let point = [[0.499_999_999_999_948_5, 0.249_999_999_999_974_2, 0.]; 2];
    let original = [[0., 0.], [0.5, 0.], [1., 1.]];
    let half = super::super::DyadicBound::from_f64(0.5).unwrap();
    let request = query(&point, &original, 0.);
    let decision =
        certify_planar_residual_on_bounds(request, [half.clone(), half.clone()], || false).unwrap();
    assert!(matches!(
        decision,
        ResidualDecision::Exceeds {
            witness: ResidualViolation::Endpoint(_)
        }
    ));
    let request = query(&point, &original, 1e-8);
    assert!(matches!(
        certify_planar_residual_on_bounds(request, [half.clone(), half], || false).unwrap(),
        ResidualDecision::Within { .. }
    ));
}
#[test]
fn exact_bounds_cannot_be_admitted_against_an_approximate_cut_trim() {
    let point = [[0.5, 0.25, 0.]; 2];
    let original = [[0., 0.], [0.5, 0.], [1., 1.]];
    let mut request = query(&point, &original, 0.);
    request.interval = [0., 0.499_999_999_999_961_3];
    let half = super::super::DyadicBound::from_f64(0.5).unwrap();
    let cause =
        certify_planar_residual_on_bounds(request, [half.clone(), half], || false).unwrap_err();
    assert!(matches!(
        cause,
        ResidualError::ExactDomain {
            declared: [0., 0.499_999_999_999_961_3],
            ..
        }
    ));
    assert!(cause.to_string().contains("declared source trim"));
}
#[test]
fn algebraic_root_bounds_narrower_than_binary64_keep_exact_domain_admission() {
    let roots = narrow_algebraic_roots();
    let bounds = roots.roots()[0].bounds();
    assert_eq!(
        roots.roots()[0].width().compare_f64(1e-20).unwrap(),
        std::cmp::Ordering::Less
    );
    let curve = [[0., 0., 0.], [1., 0., 0.]];
    let planar = [[0., 0.], [1., 0.]];
    assert!(matches!(
        certify_planar_residual_on_bounds(
            query(&curve, &planar, 0.),
            [bounds.0.clone(), bounds.1.clone()],
            || false
        )
        .unwrap(),
        ResidualDecision::Within { .. }
    ));
}

fn assert_control_array_diagnostic(cause: &ResidualError) {
    assert!(cause.to_string().contains("poles=2, weights=Some(1)"));
    assert!(cause.to_string().contains("expected"));
}

fn narrow_algebraic_roots() -> super::super::RootIsolation {
    super::super::isolate_polynomial_roots(
        &[-0.5, 0., 1.],
        [0., 1.],
        RootIsolationOptions::new(1e-20),
        || false,
    )
    .unwrap()
}

#[test]
fn reversed_exact_bounds_are_rejected_without_domain_repair() {
    let curve = [[0., 0., 0.], [1., 0., 0.]];
    let planar = [[0., 0.], [1., 0.]];
    let first = super::super::DyadicBound::from_f64(0.75).unwrap();
    let last = super::super::DyadicBound::from_f64(0.25).unwrap();
    let bounds = [first, last];
    let cause =
        certify_planar_residual_on_bounds(query(&curve, &planar, 0.), bounds.clone(), || false)
            .unwrap_err();
    assert!(
        matches!(cause, ResidualError::ExactDomain { supplied, declared: [0., 1.] } if supplied == bounds)
    );
}

#[test]
fn invalid_root_width_preserves_original_policy_failure() {
    let curve = [[0., 0., 0.], [1., 0., 0.]];
    let planar = [[0., 0.], [1., 0.]];
    let mut request = query(&curve, &planar, 0.);
    request.options = RootIsolationOptions::new(0.);
    assert!(matches!(
        certify_planar_residual(request, || false),
        Err(ResidualError::Roots(RootIsolationError::Width(0.)))
    ));
}

#[test]
fn bezier_trim_outside_declared_unit_domain_has_actual_expected_diagnostic() {
    let curve = [[0., 0., 0.], [1., 0., 0.]];
    let planar = [[0., 0.], [1., 0.]];
    let mut request = query(&curve, &planar, 0.);
    request.interval = [-0.25, 1.];
    let cause = certify_planar_residual(request, || false).unwrap_err();
    assert!(matches!(cause, ResidualError::Domain([-0.25, 1.])));
    assert!(cause.to_string().contains("[-0.25, 1.0]"));
    assert!(cause.to_string().contains("within [0,1]"));
}
