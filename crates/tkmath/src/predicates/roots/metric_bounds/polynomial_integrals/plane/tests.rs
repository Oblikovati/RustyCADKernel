// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use PolynomialPlaneFailure as PlaneFailure;

fn plane_report(
    curves: &[OrientedMetricCurve<'_>],
    support: MetricEmbedding<2>,
) -> CertifiedPolynomialPlaneMeasure {
    certify_polynomial_plane_measure(curves, support, placement(), Default::default(), || false)
        .unwrap()
}
fn assert_squared_enclosure(report: &CertifiedPolynomialPlaneMeasure, square: &ExactRatio) {
    let [low, high] = report.enclosure().map(ExactRatio::from_finite);
    assert!(low.multiply(&low) <= *square && *square <= high.multiply(&high));
    assert!(report.enclosure()[0] >= 0.);
    let width = high.subtract(&low);
    assert_eq!(
        ExactRatio::from_finite(report.absolute_error_bound()),
        width
    );
    assert_eq!(report.representative(), report.enclosure()[0]);
    assert_eq!(report.requested_options(), report.effective_options());
    assert_eq!(
        report.evidence(),
        PolynomialPlaneEvidence::CertifiedAlgebraicBound
    );
    assert_adjacent(report.enclosure());
}
fn assert_adjacent(endpoints: [f64; 2]) {
    if endpoints[0] != endpoints[1] {
        assert_eq!(endpoints[0].to_bits() + 1, endpoints[1].to_bits());
    }
}
#[test]
fn independent_scaled_cubic_uses_world_area_units() {
    let mut support = frame();
    support.linear = [[2., 0.], [0., 3.], [0., 0.]];
    let report = plane_report(&occurrences("cubic_ccw"), support);
    assert_squared_enclosure(&report, &rational("5184", "25"));
    let sweep = certify_polynomial_sweep_integral(
        &occurrences("cubic_ccw"),
        support,
        placement(),
        [0., 0., 2.],
        Default::default(),
        || false,
    )
    .unwrap();
    assert_exact_enclosure(&sweep, &rational("144", "5"));
}
#[test]
fn independent_oblique_gram_determinant_is_338() {
    let mut support = frame();
    support.linear = [[2., 7.], [3., 11.], [5., 13.]];
    // FLINT augmented-Gram determinant S=338; native Green integral I=12/5.
    let report = plane_report(&occurrences("cubic_ccw"), support);
    assert_squared_enclosure(&report, &rational("48672", "25"));
}
#[test]
fn stored_nearly_unit_axes_are_not_assumed_exactly_unit() {
    let mut support = frame();
    support.linear = [[0.6, 0.], [0., 1.], [0.8, 0.]];
    assert_eq!(0.6_f64 * 0.6 + 0.8 * 0.8, 1.);
    let square = rational(
        "81129638414606685298668707040461",
        "81129638414606681695789005144064",
    );
    let report = plane_report(&square_occurrences(), support);
    assert_squared_enclosure(&report, &square);
    assert_eq!(report.enclosure(), [1., f64::from_bits(1f64.to_bits() + 1)]);
}
#[test]
fn translations_reflections_and_traversal_preserve_unsigned_area() {
    let original = plane_report(&occurrences("cubic_ccw"), frame());
    let mut support = frame();
    support.linear[0][0] = -1.;
    support.origin = [f64::MAX, -f64::MAX, 73.];
    let mut placed = placement();
    placed.origin = [-f64::MAX, f64::MAX, -113.];
    let reversed = certify_polynomial_plane_measure(
        &occurrences("cubic_cw"),
        support,
        placed,
        Default::default(),
        || false,
    )
    .unwrap();
    assert_eq!(original, reversed);
}
#[test]
fn zero_budget_accepts_exact_square_and_refuses_algebraic_width() {
    let options = PolynomialIntegralOptions {
        absolute_error_limit: 0.,
        ..Default::default()
    };
    let exact = plane_attempt(&square_occurrences(), options, || false).unwrap();
    assert_eq!(exact.enclosure(), [1., 1.]);
    assert_eq!(exact.absolute_error_bound(), 0.);
    let error = plane_attempt(&occurrences("cubic_ccw"), options, || false).unwrap_err();
    assert_eq!(error.requested_options, options);
    assert!(
        matches!(error.cause, PlaneFailure::ErrorLimit { requested: 0., achieved } if achieved > 0.)
    );
    assert!(error.to_string().contains("world-area cap"));
}
#[test]
fn zero_integral_and_singular_support_are_scalars_without_validity_claims() {
    let mut singular = frame();
    singular.linear = [[1., 1.], [2., 2.], [3., 3.]];
    let report = plane_report(&occurrences("cubic_ccw"), singular);
    assert_eq!(report.enclosure(), [0., 0.]);
    assert_eq!(
        plane_report(&occurrences("point"), frame()).enclosure(),
        [0., 0.]
    );
    assert_eq!(
        plane_report(&occurrences("open_line"), frame()).enclosure(),
        [0.5, 0.5]
    );
}
#[test]
fn separate_public_plane_types_are_send_sync() {
    fn check<T: Send + Sync>() {}
    check::<CertifiedPolynomialPlaneMeasure>();
    check::<PolynomialPlaneEvidence>();
    check::<PolynomialPlaneError>();
    check::<PolynomialPlaneFailure>();
}

#[path = "extreme_tests.rs"]
mod extreme_tests;
#[path = "failure_tests.rs"]
mod failure_tests;

fn plane_attempt(
    curves: &[OrientedMetricCurve<'_>],
    options: PolynomialIntegralOptions,
    cancelled: impl FnMut() -> bool,
) -> Result<CertifiedPolynomialPlaneMeasure, PolynomialPlaneError> {
    certify_polynomial_plane_measure(curves, frame(), placement(), options, cancelled)
}
