// SPDX-License-Identifier: GPL-2.0-only
//! Algebra-only extreme operands do not claim an admitted live prism for those placements.
use super::*;
fn factors(q: i8, r: i8, j: i8) -> VolumeSignFactors {
    VolumeSignFactors {
        candidate_facing: q * r,
        boundary_traversal: r,
        selected_winding: 1,
        increasing_parameter: q,
        jacobian: j,
    }
}
#[test]
fn exact_winding_traversal_jacobian_product_uses_all_factors() {
    for (q, r, j, expected) in [
        (1, 1, 1, 1),
        (-1, 1, 1, -1),
        (1, -1, 1, -1),
        (-1, -1, 1, 1),
        (1, 1, -1, -1),
        (-1, 1, -1, 1),
        (1, -1, -1, 1),
        (-1, -1, -1, -1),
    ] {
        assert_eq!(
            transport::signed_integral_sign(factors(q, r, j)),
            Ok(expected)
        );
    }
    reject_sign_factors();
}
fn reject_sign_factors() {
    let mut invalid = factors(1, 1, 1);
    invalid.candidate_facing = -1;
    assert_eq!(
        transport::signed_integral_sign(invalid),
        Err(JoinedPrismVolumeCause::SignFactors(invalid))
    );
    invalid = factors(1, 1, 0);
    assert_eq!(
        transport::signed_integral_sign(invalid),
        Err(JoinedPrismVolumeCause::SignFactors(invalid))
    );
}
#[test]
fn signed_zero_underflow_bounds_remain_valid_without_metric_sign_inference() {
    let tiny = f64::from_bits(1);
    assert_eq!(
        transport::transport_enclosure([-tiny, -0.], -1),
        Ok([0., tiny])
    );
    assert_eq!(
        transport::transport_enclosure([0., tiny], 1),
        Ok([0., tiny])
    );
    reject_metric_contradictions();
}
fn reject_metric_contradictions() {
    for (bounds, sign) in [
        ([1., 2.], -1),
        ([-1., 1.], 1),
        ([2., 1.], 1),
        ([0., f64::INFINITY], 1),
        ([0., 1.], 0),
    ] {
        assert!(matches!(
            transport::transport_enclosure(bounds, sign),
            Err(JoinedPrismVolumeCause::ScalarContradiction { .. })
        ));
    }
}
fn embedding<const D: usize>(linear: [[f64; D]; 3], scale: f64) -> MetricEmbedding<D> {
    MetricEmbedding {
        linear,
        origin: [0.; 3],
        scale,
    }
}
fn extreme_scalar(height: f64, scale: f64) -> CertifiedPolynomialMeasure {
    sweep_scalar(height, scale, [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]])
}
fn sweep_scalar(height: f64, scale: f64, linear: [[f64; 3]; 3]) -> CertifiedPolynomialMeasure {
    let poles = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let curves = [OrientedMetricCurve {
        curve: MetricCurve::Bezier(MetricBezier {
            poles: &poles,
            weights: None,
        }),
        reversed: false,
    }];
    let frame = embedding([[1., 0.], [0., 1.], [0., 0.]], scale);
    let placement = embedding(linear, 1.);
    certify_polynomial_sweep_integral(
        &curves,
        frame,
        placement,
        [0., 0., height],
        PolynomialIntegralOptions::default(),
        || false,
    )
    .unwrap()
}
#[test]
fn reflected_nonunit_placement_has_independent_absolute_determinant_volume() {
    let source = sweep_scalar(2., 1., [[-2., 0., 0.], [0., 3., 0.], [0., 0., 4.]]);
    // Placed support columns (-2,0,0),(0,3,0) and world extrusion (0,0,2): J=-12.
    // The independent positive integral is (12/5)*12 = 144/5.
    let (bounds, representative) = transport::transport_scalar(&source, -1).unwrap();
    assert_eq!(
        bounds,
        [
            f64::from_bits(0x403ccccccccccccc),
            f64::from_bits(0x403ccccccccccccd)
        ]
    );
    assert_eq!(representative, bounds[1]);
    assert_eq!(source.representative(), -representative);
}
#[test]
fn underflow_represents_upper_endpoint_without_reducing_certified_error() {
    let tiny = f64::from_bits(1);
    // Independent exact quantity is (12/5)*(1/4)^2*tiny = 3*tiny/20.
    let source = extreme_scalar(-tiny, 0.25);
    assert_eq!(source.enclosure(), [-tiny, -0.]);
    assert_eq!(source.absolute_error_bound(), tiny);
    let (bounds, representative) = transport::transport_scalar(&source, -1).unwrap();
    assert_eq!(bounds, [0., tiny]);
    assert_eq!(representative, tiny);
    assert_eq!(
        source.requested_options(),
        PolynomialIntegralOptions::default()
    );
}
#[test]
fn nonunit_support_scale_preserves_closed_form_and_original_error() {
    for height in [-2., 2.] {
        let source = extreme_scalar(height, 2.);
        let sign = if height < 0. { -1 } else { 1 };
        let (bounds, representative) = transport::transport_scalar(&source, sign).unwrap();
        assert_eq!(
            bounds,
            [
                f64::from_bits(0x4033333333333333),
                f64::from_bits(0x4033333333333334)
            ]
        );
        assert_eq!(representative, f64::from(sign) * source.representative());
    }
}
