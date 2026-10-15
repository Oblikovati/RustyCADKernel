// SPDX-License-Identifier: GPL-2.0-only
//! Invariants, exact classification, refusals, cancellation and determinism.
use super::super::super::{RootIsolationError, certificate_arithmetic::ExactRatio};
use super::super::{
    MetricBezier, MetricCurve, PolynomialIntegralOptions, ResidualError,
    certify_polynomial_boundary_integral,
};
use super::closed_form_tests::{QUARTER_WEIGHTS, arcs, assert_contains_pi_multiple, circle_poles};
use super::*;
use RationalIntegralFailure as Failure;

const CUBIC: [[f64; 2]; 4] = [[0., 0.], [4., 0.], [0., 4.], [0., 0.]];
fn bezier<'a>(poles: &'a [[f64; 2]], weights: Option<&'a [f64]>) -> OrientedMetricCurve<'a> {
    OrientedMetricCurve {
        curve: MetricCurve::Bezier(MetricBezier { poles, weights }),
        reversed: false,
    }
}
fn boundary(curves: &[OrientedMetricCurve<'_>]) -> CertifiedRationalMeasure {
    certify_rational_boundary_integral(curves, Default::default(), || false).unwrap()
}
fn refusal(
    curves: &[OrientedMetricCurve<'_>],
    options: RationalIntegralOptions,
) -> RationalIntegralError {
    certify_rational_boundary_integral(curves, options, || false).unwrap_err()
}
fn loose() -> RationalIntegralOptions {
    RationalIntegralOptions {
        absolute_error_limit: 1e-3,
        ..Default::default()
    }
}

#[test]
fn polynomial_and_equal_weight_curves_match_the_polynomial_entry_exactly() {
    let polynomial = certify_polynomial_boundary_integral(
        &[bezier(&CUBIC, None)],
        PolynomialIntegralOptions::default(),
        || false,
    )
    .unwrap();
    for weights in [None, Some(&[3., 3., 3., 3.][..]), Some(&[0.25; 4][..])] {
        let report = boundary(&[bezier(&CUBIC, weights)]);
        assert_eq!(report.enclosure(), polynomial.enclosure());
        assert_eq!(report.representative(), polynomial.representative());
        assert_eq!(
            report.absolute_error_bound(),
            polynomial.absolute_error_bound()
        );
        assert_eq!(report.evidence(), RationalIntegralEvidence::ExactRational);
        assert_eq!(report.integration_subintervals(), 0);
    }
}
#[test]
fn mobius_reparametrization_and_weight_scaling_preserve_the_integral() {
    // w_i * lambda^i reparametrizes the same conic; a common factor changes nothing.
    let poles = circle_poles([0., 0.], 1.);
    for weights in [
        [1., 4., 32.],
        [1., 0.25, 0.125],
        [8., 8., 16.],
        [1., 64., 8192.],
    ] {
        let report = boundary(&arcs(&poles, &weights, false));
        assert_contains_pi_multiple(&report, 1.);
    }
}
#[test]
fn reversed_traversal_of_the_reversed_arc_restores_the_sector() {
    // The same quarter with poles and weights reversed, traversed backwards.
    let poles = [[0., 1.], [1., 1.], [1., 0.]];
    let weights = [2., 1., 1.];
    let mut occurrence = bezier(&poles, Some(&weights));
    occurrence.reversed = true;
    assert_contains_pi_multiple(&boundary(&[occurrence]), 0.25);
}
#[test]
fn degenerate_point_and_collinear_rational_curves_integrate_to_exact_zero() {
    let point = [[3., 4.]; 3];
    let collinear = [[1., 1.], [2., 2.], [3., 3.]];
    for poles in [&point, &collinear] {
        let options = RationalIntegralOptions {
            absolute_error_limit: 0.,
            ..Default::default()
        };
        let curves = [bezier(poles, Some(&QUARTER_WEIGHTS))];
        let report = certify_rational_boundary_integral(&curves, options, || false).unwrap();
        assert_eq!(report.enclosure(), [0., 0.]);
        assert_eq!(report.absolute_error_bound(), 0.);
        assert_eq!(
            report.evidence(),
            RationalIntegralEvidence::ErrorBoundedIntegration
        );
    }
}
#[test]
fn mixed_rational_and_line_boundary_gives_closed_form_half_disc() {
    // Upper half of the unit circle closed by the diameter: pi/2.
    let poles = circle_poles([0., 0.], 1.);
    let mut curves = arcs(&poles[..2], &QUARTER_WEIGHTS, false);
    curves.push(OrientedMetricCurve {
        curve: MetricCurve::Line {
            origin: [-1., 0.],
            direction: [2., 0.],
        },
        reversed: false,
    });
    assert_contains_pi_multiple(&boundary(&curves), 0.5);
}
#[test]
fn policy_count_degree_and_weight_refusals_are_typed() {
    let curve = [bezier(&CUBIC, None)];
    let bad = [
        RationalIntegralOptions {
            behavior_version: 2,
            ..Default::default()
        },
        RationalIntegralOptions {
            max_subintervals: 0,
            ..Default::default()
        },
        RationalIntegralOptions {
            max_subintervals: 65537,
            ..Default::default()
        },
        RationalIntegralOptions {
            absolute_error_limit: -1.,
            ..Default::default()
        },
        RationalIntegralOptions {
            absolute_error_limit: f64::NAN,
            ..Default::default()
        },
        RationalIntegralOptions {
            max_curves: 0,
            ..Default::default()
        },
    ];
    for options in bad {
        assert!(matches!(refusal(&curve, options).cause, Failure::Policy(_)));
    }
    assert!(matches!(
        refusal(&[], loose()).cause,
        Failure::EmptyBoundary
    ));
    let many = vec![curve[0]; 65];
    assert!(matches!(
        refusal(&many, loose()).cause,
        Failure::CurveCount { actual: 65, .. }
    ));
    let degree = RationalIntegralOptions {
        max_degree: 2,
        ..loose()
    };
    let error = refusal(&curve, degree);
    assert!(matches!(
        error.cause,
        Failure::Degree {
            actual: 3,
            limit: 2
        }
    ));
    assert_eq!(error.curve_index, Some(0));
    for weight in [0., -1., f64::INFINITY] {
        let weights = [1., weight, 1.];
        let error = refusal(&[bezier(&CUBIC[..3], Some(&weights))], loose());
        assert!(matches!(error.cause, Failure::Metric(_)), "{error}");
    }
}
#[test]
fn unmet_budgets_are_incomplete_without_partial_results() {
    let poles = circle_poles([0., 0.], 1.);
    let curves = arcs(&poles, &QUARTER_WEIGHTS, false);
    let starved = RationalIntegralOptions {
        max_subintervals: 2,
        ..Default::default()
    };
    let error = refusal(&curves, starved);
    assert!(
        matches!(error.cause, Failure::WorkLimit { limit: 2, .. }),
        "{error}"
    );
    assert_eq!(error.curve_index, Some(0));
    // A zero cap is unreachable for a nonconstant denominator; depth-first refinement
    // stops at the explicit work limit (kept small so the test stays fast).
    let exact_zero = RationalIntegralOptions {
        absolute_error_limit: 0.,
        max_subintervals: 64,
        ..Default::default()
    };
    assert!(matches!(
        refusal(&curves, exact_zero).cause,
        Failure::WorkLimit { .. }
    ));
    // A polynomial value that is not binary64 cannot meet a zero cap after rounding.
    let third = [[0., 0.], [1., 0.], [0., 1.], [0., 0.]];
    let error = refusal(&[bezier(&third, None)], exact_zero);
    assert!(
        matches!(error.cause, Failure::ErrorLimit { requested: 0., .. }),
        "{error}"
    );
}
#[test]
fn every_cancellation_checkpoint_refuses_and_reuse_is_clean() {
    let poles = circle_poles([0., 0.], 1.);
    let curves = arcs(&poles[..1], &QUARTER_WEIGHTS, false);
    let mut polls = 0;
    let complete = certify_rational_boundary_integral(&curves, loose(), || {
        polls += 1;
        false
    })
    .unwrap();
    assert!(polls > 3);
    for stop in 0..polls {
        let mut seen = 0;
        let result = certify_rational_boundary_integral(&curves, loose(), || {
            seen += 1;
            seen > stop
        });
        let error = result.unwrap_err();
        let Failure::Metric(cause) = &error.cause else {
            panic!("stop {stop}: {error}");
        };
        assert!(matches!(
            **cause,
            ResidualError::Roots(RootIsolationError::Cancelled)
        ));
    }
    assert_eq!(
        certify_rational_boundary_integral(&curves, loose(), || false).unwrap(),
        complete
    );
}
#[test]
fn replay_is_identical_across_calls_and_threads_and_inputs_are_untouched() {
    let poles = circle_poles([0.5, -2.], 1.5);
    let weights = [1., 3., 18.];
    let before = (poles, weights);
    let first = boundary(&arcs(&poles, &weights, false));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            std::thread::spawn(move || {
                let poles = circle_poles([0.5, -2.], 1.5);
                boundary(&arcs(&poles, &[1., 3., 18.], false))
            })
        })
        .collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), first);
    }
    assert_eq!(boundary(&arcs(&poles, &weights, false)), first);
    assert_eq!((poles, weights), before);
    assert_eq!(first.requested_options(), first.effective_options());
}
#[test]
fn singular_sweep_is_exact_zero_scalar() {
    let poles = circle_poles([0., 0.], 1.);
    let frame = MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    };
    let placement = MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    };
    let report = certify_rational_sweep_integral(
        &arcs(&poles, &QUARTER_WEIGHTS, false),
        frame,
        placement,
        [1., 1., 0.],
        Default::default(),
        || false,
    )
    .unwrap();
    assert_eq!(report.enclosure(), [0., 0.]);
    assert_eq!(report.absolute_error_bound(), 0.);
}
/// Exact certified interval `factor * [rep - err, rep + err]`.
fn certified(report: &CertifiedRationalMeasure, factor: f64) -> [ExactRatio; 2] {
    let center = ExactRatio::from_finite(report.representative());
    let error = ExactRatio::from_finite(report.absolute_error_bound());
    let factor = ExactRatio::from_finite(factor);
    [
        center.subtract(&error).multiply(&factor),
        center.add(&error).multiply(&factor),
    ]
}
/// Certified intervals of the same exact value must overlap.
fn assert_overlapping(left: [ExactRatio; 2], right: [ExactRatio; 2]) {
    assert!(
        left[0] <= right[1] && right[0] <= left[1],
        "{left:?} vs {right:?}"
    );
}
fn moved(poles: &[[f64; 2]], motion: fn([f64; 2]) -> [f64; 2]) -> Vec<[f64; 2]> {
    poles.iter().copied().map(motion).collect()
}
fn high_degree_loop() -> (Vec<[f64; 2]>, Vec<f64>) {
    let mut poles: Vec<[f64; 2]> = (0..16)
        .map(|i| {
            let angle = f64::from(i) * 0.39;
            [
                (4. * angle.cos()).round() / 4.,
                (4. * angle.sin()).round() / 4.,
            ]
        })
        .collect();
    poles.push(poles[0]);
    let weights = (0..17).map(|i| 1. + f64::from(i % 5) * 0.75).collect();
    (poles, weights)
}
#[test]
fn arbitrary_rational_loops_are_invariant_under_exact_motions() {
    // Closed loops: a rational cubic and a degree-16 curve with unequal dyadic weights.
    let cubic = vec![[0., 0.], [4., 0.5], [-1., 3.], [0., 0.]];
    let cubic_weights = vec![1., 0.375, 5., 2.];
    for (poles, weights) in [(cubic, cubic_weights), high_degree_loop()] {
        let measure = |poles: &[[f64; 2]]| boundary(&[bezier(poles, Some(&weights))]);
        let base = measure(&poles);
        let translated = measure(&moved(&poles, |[x, y]| [x + 3., y - 7.]));
        let rotated = measure(&moved(&poles, |[x, y]| [-y, x]));
        let doubled = measure(&moved(&poles, |[x, y]| [2. * x, 2. * y]));
        assert_overlapping(certified(&base, 1.), certified(&translated, 1.));
        assert_overlapping(certified(&base, 1.), certified(&rotated, 1.));
        assert_overlapping(certified(&base, 4.), certified(&doubled, 1.));
        assert_eq!(
            base.evidence(),
            RationalIntegralEvidence::ErrorBoundedIntegration
        );
    }
}
