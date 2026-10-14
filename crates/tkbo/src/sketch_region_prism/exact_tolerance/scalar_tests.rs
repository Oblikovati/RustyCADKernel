// SPDX-License-Identifier: GPL-2.0-only
//! Independent rounding and singularity traps distinguish represented geometry from rounded points.
use super::{frames::ToleranceFrameFailure, scalar::*, test_support::*};
use crate::sketch_region_source_conformance::{
    certify_captured_endpoint, certify_captured_placed_endpoint,
};
use rustycad_tkmath::predicates::roots::{DyadicBound, metric_bounds::*};

#[test]
fn exact_raw_transport_refuses_the_two_to_fifty_three_rounding_trap() {
    let shift = 9_007_199_254_740_992.;
    let curve = constant_curve([shift, 0., 0.]);
    let mut placement = identity();
    placement.origin = [shift, 0., 0.];
    assert_eq!(shift + 1., shift);
    assert_within(
        certify_captured_endpoint(&curve, 0., [shift, 0., 0.], 0.),
        0.,
    );
    assert_exceeded(certify_exact_endpoint(&curve, 0., [1., 0., 0.], placement));
    let threshold = certify_captured_placed_endpoint(&curve, 0., [1., 0., 0.], placement, 1.);
    assert_within(threshold, 1.);
}

#[test]
fn singular_transform_cannot_hide_different_local_vertices() {
    let curve = constant_curve([0.; 3]);
    let mut placement = identity();
    placement.linear[0] = [0.; 3];
    assert_within(
        certify_captured_placed_endpoint(&curve, 0., [1., 0., 0.], placement, 0.),
        0.,
    );
    let failure = certify_exact_endpoint(&curve, 0., [1., 0., 0.], placement)
        .err()
        .unwrap();
    assert!(matches!(
        failure,
        ExactEndpointFailure::Frame {
            role: EndpointFrameRole::VertexOccurrence,
            cause: ToleranceFrameFailure::Singular { .. }
        }
    ));
}

#[test]
fn noncommuting_rotation_scale_translation_has_independent_exact_expected_point() {
    let curve = constant_curve([-1., 7., 13.]);
    let mut placement = identity();
    placement.linear = [[0., -1., 0.], [1., 0., 0.], [0., 0., 1.]];
    placement.scale = 2.;
    placement.origin = [3., 5., 7.];
    let certificate = certify_exact_endpoint(&curve, 0., [1., 2., 3.], placement)
        .ok()
        .unwrap();
    assert_zero_certificate(&certificate);
    let wrong = constant_curve([-14., 8., 20.]);
    assert_exceeded(certify_exact_endpoint(&wrong, 0., [1., 2., 3.], placement));
}

#[test]
fn invalid_curve_frame_keeps_original_operand_role() {
    let mut curve = constant_curve([0.; 3]);
    curve.placement.scale = 0.;
    assert!(matches!(
        certify_exact_endpoint(&curve, 0., [0.; 3], identity()),
        Err(ExactEndpointFailure::Frame {
            role: EndpointFrameRole::CurveRepresentation,
            cause: ToleranceFrameFailure::Scale { actual: 0. }
        })
    ));
}

#[test]
fn invalid_endpoint_parameter_keeps_original_metric_cause() {
    let curve = constant_curve([0.; 3]);
    let failure = certify_exact_endpoint(&curve, f64::NAN, [0.; 3], identity())
        .err()
        .unwrap();
    assert!(matches!(failure, ExactEndpointFailure::Residual(report)
        if matches!(report.decision, Err(ResidualError::Value {
            role: "parameter endpoint", index: 0, value
        }) if value.is_nan())));
}

#[test]
fn exact_transport_retains_extreme_scale_products_without_float_underflow_or_overflow() {
    for scale in [f64::from_bits(1), -f64::from_bits(1), f64::MAX, -f64::MAX] {
        let mut curve = constant_curve([1., 2., 3.]);
        curve.placement.scale = scale;
        let coefficient = if scale.abs() < 1. { 0.5 } else { 2. };
        curve.placement.linear = [
            [coefficient, 0., 0.],
            [0., coefficient, 0.],
            [0., 0., coefficient],
        ];
        let result = certify_exact_endpoint(&curve, 0., [1., 2., 3.], curve.placement);
        assert_zero_certificate(&result.ok().unwrap());
    }
}

#[test]
fn nonfinite_raw_vertex_coordinate_retains_exact_metric_value_cause() {
    let invalid = f64::from_bits(0x7ff8_0000_0000_9876);
    let result =
        certify_exact_endpoint(&constant_curve([0.; 3]), 0., [invalid, 0., 0.], identity());
    let ExactEndpointFailure::Residual(report) = result.err().unwrap() else {
        panic!("metric cause")
    };
    let Err(ResidualError::Value { value, .. }) = report.decision else {
        panic!("value cause")
    };
    assert_eq!(value.to_bits(), invalid.to_bits());
}

fn assert_zero_certificate(certificate: &ExactEndpointCertificate) {
    assert_eq!(
        certificate.proven_squared_upper,
        DyadicBound::from_f64(0.).unwrap()
    );
    assert_eq!(certificate.requested_spatial, 0.);
    assert_eq!(certificate.effective_spatial, 0.);
    assert_eq!(certificate.bisections, 0);
}

fn assert_within(report: TransportedResidualReport, squared: f64) {
    assert!(
        matches!(report.decision, Ok(ResidualDecision::Within { proven_squared_upper })
        if proven_squared_upper == DyadicBound::from_f64(squared).unwrap())
    );
}

fn assert_exceeded(result: Result<ExactEndpointCertificate, ExactEndpointFailure>) {
    let failure = result.err().unwrap();
    assert!(matches!(failure, ExactEndpointFailure::Residual(report)
        if matches!(report.decision, Ok(ResidualDecision::Exceeds { .. }))));
}
