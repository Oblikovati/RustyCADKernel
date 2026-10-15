// SPDX-License-Identifier: GPL-2.0-only
//! Independent closed forms: exact rational unit-circle quarters give pi-multiples whose
//! 40-digit rational bounds must lie inside `representative +- certified error`.
use super::super::super::certificate_arithmetic::ExactRatio;
use super::super::{MetricBezier, MetricCurve, MetricEmbedding};
use super::*;

/// x = (1-t^2)/(1+t^2), y = 2t/(1+t^2): exact poles and weights, no rounded sqrt(2).
pub(super) const QUARTER_WEIGHTS: [f64; 3] = [1., 1., 2.];
/// The four quarters of the circle with `radius` about `center`, counterclockwise.
pub(super) fn circle_poles(center: [f64; 2], radius: f64) -> [[[f64; 2]; 3]; 4] {
    let unit = [
        [[1., 0.], [1., 1.], [0., 1.]],
        [[0., 1.], [-1., 1.], [-1., 0.]],
        [[-1., 0.], [-1., -1.], [0., -1.]],
        [[0., -1.], [1., -1.], [1., 0.]],
    ];
    unit.map(|arc| arc.map(|[x, y]| [center[0] + radius * x, center[1] + radius * y]))
}
pub(super) fn arcs<'a>(
    poles: &'a [[[f64; 2]; 3]],
    weights: &'a [f64],
    reversed: bool,
) -> Vec<OrientedMetricCurve<'a>> {
    let occurrence = |arc: &'a [[f64; 2]; 3]| OrientedMetricCurve {
        curve: MetricCurve::Bezier(MetricBezier {
            poles: arc,
            weights: Some(weights),
        }),
        reversed,
    };
    poles.iter().map(occurrence).collect()
}
/// pi lies strictly between these 40-digit decimal rationals.
pub(super) fn pi_bounds() -> [ExactRatio; 2] {
    let digits: num_bigint::BigInt = "31415926535897932384626433832795028841971".parse().unwrap();
    let scale = num_bigint::BigInt::from(10).pow(40);
    [
        ExactRatio::new(digits.clone(), scale.clone()),
        ExactRatio::new(digits + 1, scale),
    ]
}
/// Assert `[rep - err, rep + err]` contains `factor * pi` and the error meets the cap.
pub(super) fn assert_contains_pi_multiple(report: &CertifiedRationalMeasure, factor: f64) {
    let factor = ExactRatio::from_finite(factor);
    let [low, high] = pi_bounds().map(|bound| bound.multiply(&factor));
    let (low, high) = if low <= high {
        (low, high)
    } else {
        (high, low)
    };
    let representative = ExactRatio::from_finite(report.representative());
    let error = ExactRatio::from_finite(report.absolute_error_bound());
    assert!(
        representative.subtract(&error) <= low,
        "{report:?} below {low:?}"
    );
    assert!(
        representative.add(&error) >= high,
        "{report:?} above {high:?}"
    );
    let [lower, upper] = report.enclosure().map(ExactRatio::from_finite);
    assert!(lower <= low && high <= upper, "{report:?} enclosure");
    assert!(report.absolute_error_bound() <= report.requested_options().absolute_error_limit);
    assert_eq!(
        report.evidence(),
        RationalIntegralEvidence::ErrorBoundedIntegration
    );
}
fn boundary(curves: &[OrientedMetricCurve<'_>]) -> CertifiedRationalMeasure {
    certify_rational_boundary_integral(curves, Default::default(), || false).unwrap()
}
fn identity_frame() -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
fn identity_placement() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}

#[test]
fn quarter_arc_sector_is_pi_over_four() {
    let poles = circle_poles([0., 0.], 1.);
    assert_contains_pi_multiple(&boundary(&arcs(&poles[..1], &QUARTER_WEIGHTS, false)), 0.25);
}
#[test]
fn full_circles_give_pi_r_squared_independent_of_center() {
    for (center, radius) in [([0., 0.], 1.), ([2., 5.], 3.), ([-7.5, 0.25], 0.125)] {
        let poles = circle_poles(center, radius);
        let report = boundary(&arcs(&poles, &QUARTER_WEIGHTS, false));
        assert_contains_pi_multiple(&report, radius * radius);
        let reversed = boundary(&arcs(&poles, &QUARTER_WEIGHTS, true));
        assert_contains_pi_multiple(&reversed, -radius * radius);
    }
}
#[test]
fn ellipse_from_axis_scaling_gives_pi_a_b() {
    let poles = circle_poles([0., 0.], 1.).map(|arc| arc.map(|[x, y]| [4. * x, 0.75 * y]));
    assert_contains_pi_multiple(&boundary(&arcs(&poles, &QUARTER_WEIGHTS, false)), 3.);
}
#[test]
fn sweep_of_circle_is_pi_r_squared_h_with_sign_of_extrusion() {
    let poles = circle_poles([1., 1.], 2.);
    let curves = arcs(&poles, &QUARTER_WEIGHTS, false);
    for height in [2., -0.5, 1024.] {
        let report = certify_rational_sweep_integral(
            &curves,
            identity_frame(),
            identity_placement(),
            [0.25, -3., height],
            Default::default(),
            || false,
        )
        .unwrap();
        assert_contains_pi_multiple(&report, 4. * height);
        assert_eq!(
            report.operation(),
            PolynomialIntegralOperation::SignedSweepIntegral
        );
    }
}
#[test]
fn looser_cap_needs_no_more_work_and_still_contains_closed_form() {
    let poles = circle_poles([0., 0.], 1.);
    let curves = arcs(&poles, &QUARTER_WEIGHTS, false);
    let mut previous = u64::MAX;
    for cap in [1e-14, 1e-12, 1e-8, 1e-4, 1e-1] {
        let options = RationalIntegralOptions {
            absolute_error_limit: cap,
            ..Default::default()
        };
        let report = certify_rational_boundary_integral(&curves, options, || false).unwrap();
        assert_contains_pi_multiple(&report, 1.);
        assert!(report.integration_subintervals() <= previous);
        previous = report.integration_subintervals();
    }
}
