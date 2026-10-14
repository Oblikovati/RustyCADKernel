// SPDX-License-Identifier: GPL-2.0-only
use super::super::MetricBezier;
use super::*;

pub(super) fn identity() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
pub(super) fn curve(poles: &[[f64; 3]]) -> MetricCurve<'_, 3> {
    MetricCurve::Bezier(MetricBezier {
        poles,
        weights: None,
    })
}
pub(super) fn integrate(
    poles: &[[f64; 3]],
    options: PolynomialSideOptions,
) -> Result<CertifiedPolynomialSideIntegral, PolynomialSideError> {
    certify_polynomial_side_integral(
        curve(poles),
        identity(),
        identity(),
        [0., 0., 1.],
        options,
        || false,
    )
}
fn assert_contains(report: &CertifiedPolynomialSideIntegral, expected: ExactRatio) {
    let [lower, upper] = report.enclosure();
    assert!(
        ExactRatio::from_finite(lower) <= expected && expected <= ExactRatio::from_finite(upper)
    );
    assert_eq!(report.representative(), lower);
    assert_eq!(report.evidence(), PolynomialSideEvidence::CertifiedBound);
    assert!(report.absolute_error_bound() <= report.requested_options().absolute_error_limit);
    assert_eq!(report.requested_options(), report.effective_options());
}
#[test]
fn nonconstant_ph_cubic_has_independent_exact_integral_four() {
    let poles = [[0., 0., 0.], [1., 0., 0.], [2., 1., 0.], [2., 3., 0.]];
    let report = integrate(&poles, Default::default()).unwrap();
    assert_contains(&report, ExactRatio::from_finite(4.));
    record_work("ph_cubic", &report);
}
#[test]
fn reversing_quadratic_has_exact_five_thirds_despite_nondyadic_zero_derivative() {
    let poles = [[0., 0., 0.], [-1., 0., 0.], [1., 0., 0.]];
    let report = integrate(&poles, Default::default()).unwrap();
    assert_contains(&report, ExactRatio::new(5.into(), 3.into()));
    record_work("reversing_quadratic", &report);
}
#[test]
fn active_closed_cubic_completes_conditional_measure_without_face_claim() {
    let poles = [[0., 0., 0.], [4., 0., 0.], [0., 4., 0.], [0., 0., 0.]];
    let report = certify_polynomial_side_integral(
        curve(&poles),
        identity(),
        identity(),
        [0., 0., 2.],
        Default::default(),
        || false,
    )
    .unwrap();
    assert!(report.enclosure()[0] > 0.);
    assert!(report.absolute_error_bound() <= report.requested_options().absolute_error_limit);
    record_work("closed_cubic_height_two", &report);
}
#[test]
fn line_parallel_and_zero_extrusion_have_closed_form_results() {
    for (extrusion, area) in [([0., 0., 2.], 10.), ([3., 4., 0.], 0.), ([0.; 3], 0.)] {
        let report = exact_line(extrusion);
        assert_eq!(report.enclosure(), [area; 2]);
        assert_eq!(
            report.work(),
            PolynomialSideWork {
                subdivisions: 0,
                interval_bounds: 1
            }
        );
    }
}
fn exact_line(extrusion: [f64; 3]) -> CertifiedPolynomialSideIntegral {
    let options = PolynomialSideOptions {
        absolute_error_limit: 0.,
        max_subdivisions: 0,
        ..Default::default()
    };
    let line = MetricCurve::Line {
        origin: [5., 6., 7.],
        direction: [3., 4., 0.],
    };
    certify_polynomial_side_integral(line, identity(), identity(), extrusion, options, || false)
        .unwrap()
}
#[test]
fn constant_curve_has_completed_zero_integral() {
    let report = integrate(&[[2., 3., 4.]], Default::default()).unwrap();
    assert_eq!(report.enclosure(), [0.; 2]);
}
fn record_work(fixture: &str, report: &CertifiedPolynomialSideIntegral) {
    println!(
        "{{\"fixture\":\"{fixture}\",\"subdivisions\":{},\"interval_bounds\":{},\"bounds\":{:?},\"error\":{}}}",
        report.work().subdivisions,
        report.work().interval_bounds,
        report.enclosure(),
        report.absolute_error_bound()
    );
}
#[test]
fn all_public_scalar_types_are_send_sync() {
    fn scalar<T: Send + Sync>() {
        // The bound is compile-time evidence; this assertion keeps no operation state.
        assert!(std::mem::size_of::<T>() < usize::MAX);
    }
    scalar::<PolynomialSideOptions>();
    scalar::<PolynomialSideWork>();
    scalar::<PolynomialSideEvidence>();
    scalar::<PolynomialSideError>();
    scalar::<PolynomialSideFailure>();
    scalar::<CertifiedPolynomialSideIntegral>();
}
