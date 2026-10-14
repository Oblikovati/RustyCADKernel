// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[test]
fn opposing_extreme_scales_remain_exact_until_final_sqrt() {
    for (native, world, expected) in [(-500, 600, 200), (500, -600, -200)] {
        let poles = CUBIC.map(|point| point.map(|x| x * 2f64.powi(native)));
        let mut support = frame();
        support.scale = 2f64.powi(world);
        let options = unrestricted_rounding_budget();
        let report = certify_polynomial_plane_measure(
            &[polynomial(&poles)],
            support,
            placement(),
            options,
            || false,
        )
        .unwrap();
        let area = rational("12", "5").multiply(&ExactRatio::from_finite(2f64.powi(expected)));
        assert_squared_enclosure(&report, &area.multiply(&area));
    }
}
#[test]
fn subnormal_area_enclosure_retains_nonzero_exact_square() {
    let mut support = frame();
    support.scale = 2f64.powi(-600);
    let report = plane_report(&square_occurrences(), support);
    assert_eq!(report.enclosure(), [0., f64::from_bits(1)]);
    assert_eq!(report.absolute_error_bound(), f64::from_bits(1));
    let area =
        ExactRatio::from_finite(support.scale).multiply(&ExactRatio::from_finite(support.scale));
    assert_squared_enclosure(&report, &area.multiply(&area));
}
#[test]
fn final_unrepresentable_area_preserves_exact_squared_diagnostic() {
    let mut support = frame();
    support.scale = f64::MAX;
    let error = certify_polynomial_plane_measure(
        &square_occurrences(),
        support,
        placement(),
        Default::default(),
        || false,
    )
    .unwrap_err();
    assert!(
        matches!(&error.cause, PlaneFailure::UnrepresentableSquaredMeasure(square) if !square.is_empty())
    );
    assert!(error.to_string().contains("finite square-root enclosure"));
}
#[test]
fn affine_placement_composes_columns_before_gram_determinant() {
    let mut support = frame();
    support.linear = [[1., 0.], [0., 1.], [1., 1.]];
    support.scale = 2.;
    let mut placed = placement();
    placed.linear = [[2., 0., 1.], [0., 3., 0.], [0., 0., 1.]];
    placed.scale = 0.5;
    let report = certify_polynomial_plane_measure(
        &square_occurrences(),
        support,
        placed,
        Default::default(),
        || false,
    )
    .unwrap();
    // Placed columns (3,0,1),(1,3,1), cross=(-3,-2,9), squared norm94.
    assert_squared_enclosure(&report, &rational("94", "1"));
}

fn unrestricted_rounding_budget() -> PolynomialIntegralOptions {
    PolynomialIntegralOptions {
        absolute_error_limit: f64::MAX,
        ..Default::default()
    }
}

#[test]
fn generated_dyadic_supports_preserve_exact_area_and_degeneracy() {
    for exponent in [-1074, -1000, -500, -2, 0, 2, 500, 1000, 1023] {
        let scale = f64::from_bits(if exponent < -1022 {
            1 << (exponent + 1074)
        } else {
            ((exponent + 1023) as u64) << 52
        });
        assert_scaled_columns(scale);
    }
}
fn assert_scaled_columns(scale: f64) {
    let mut support = frame();
    support.linear[0][0] = scale;
    let report = plane_report(&square_occurrences(), support);
    assert_eq!(report.enclosure(), [scale, scale]);
    assert_eq!(report.absolute_error_bound(), 0.);
    support.linear = [[scale, scale], [scale, scale], [scale, scale]];
    assert_eq!(
        plane_report(&square_occurrences(), support).enclosure(),
        [0., 0.]
    );
}
#[test]
fn gram_cancellation_is_exact_when_float_products_lose_unit_determinant() {
    let n = 4_503_599_627_370_496.;
    let mut support = frame();
    support.linear = [[n, n + 1.], [n - 1., n], [0., 0.]];
    assert_eq!(n * n - (n - 1.) * (n + 1.), 0.);
    let report = plane_report(&square_occurrences(), support);
    assert_eq!(report.enclosure(), [1., 1.]);
}
