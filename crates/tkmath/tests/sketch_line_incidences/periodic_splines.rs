// SPDX-License-Identifier: GPL-2.0-only
//! Native periodic B-splines (ADR 0080) certify exactly as their hand-unwrapped
//! flat-knot form. The expected flat knots below repeat each knot by its
//! multiplicity and extend one period on each side (`BSplCLib::KnotSequence`).
use super::*;

const HEXAGON: [[f64; 2]; 6] = [
    [2., 0.],
    [1., 2.],
    [-1., 2.],
    [-2., 0.],
    [-1., -2.],
    [1., -2.],
];

fn periodic(
    degree: usize,
    poles: &[[f64; 2]],
    knots: &[f64],
    multiplicities: &[usize],
    bounds: [f64; 2],
) -> SketchPeriodicSpline {
    SketchPeriodicSpline {
        degree,
        poles: poles.to_vec(),
        weights: None,
        knots: knots.to_vec(),
        multiplicities: multiplicities.to_vec(),
        bounds,
    }
}
// Cubic, simple knots 0..6: flat knots -3..9, poles p0..p5 then p0, p1, p2.
fn cubic(bounds: [f64; 2]) -> (SketchIncidenceCurve, SketchIncidenceCurve) {
    let knots = [0., 1., 2., 3., 4., 5., 6.];
    let native = periodic(3, &HEXAGON, &knots, &[1; 7], bounds);
    let flat: Vec<f64> = (-3..=9).map(f64::from).collect();
    let poles: Vec<[f64; 2]> = (0..9).map(|i| HEXAGON[i % 6]).collect();
    let manual = spline(3, &poles, &flat, bounds);
    (SketchIncidenceCurve::PeriodicSpline(native), manual)
}
// Rational quadratic with double knots: flat knots -1, 0,0,1,1,2,2,3,3,4,4, 5.
fn rational_quadratic() -> (SketchIncidenceCurve, SketchIncidenceCurve) {
    let poles: Vec<[f64; 2]> = (0..8).map(|i| HEXAGON[i % 6]).collect();
    let weights: Vec<f64> = (0..8).map(|i| [1., 0.5][i % 2]).collect();
    let mut native = periodic(2, &poles, &[0., 1., 2., 3., 4.], &[2; 5], [0.5, 4.]);
    native.weights = Some(weights.clone());
    let flat = [-1., 0., 0., 1., 1., 2., 2., 3., 3., 4., 4., 5.];
    let wrapped: Vec<[f64; 2]> = (0..9).map(|i| poles[i % 8]).collect();
    let manual = SketchIncidenceSpline {
        degree: 2,
        poles: wrapped,
        weights: Some((0..9).map(|i| weights[i % 8]).collect()),
        flat_knots: flat.to_vec(),
        bounds: [0.5, 4.],
    };
    (
        SketchIncidenceCurve::PeriodicSpline(native),
        SketchIncidenceCurve::Spline(manual),
    )
}
fn assert_same_incidences(native: &SketchLineIncidenceReport, manual: &SketchLineIncidenceReport) {
    assert_eq!(native.ordered(), manual.ordered());
    assert_eq!(native.targets().len(), manual.targets().len());
    for (left, right) in native.targets().iter().zip(manual.targets()) {
        assert_eq!(left.contacts(), right.contacts());
        assert_eq!(left.overlaps(), right.overlaps());
    }
}
#[test]
fn periodic_targets_equal_their_unwrapped_form() {
    let line = source([[-3., 0.25], [3., 0.5]], SketchLineDomain::Support);
    for (native, manual) in [cubic([0., 6.]), cubic([1.5, 4.25]), rational_quadratic()] {
        let native_report = certify(line.clone(), &[target(4, native.clone())]);
        let manual_report = certify(line.clone(), &[target(4, manual)]);
        assert!(!native_report.ordered().is_empty(), "{native_report:#?}");
        assert_same_incidences(&native_report, &manual_report);
        assert_eq!(native_report.targets()[0].target().curve, native);
    }
}
#[test]
fn periodic_sources_equal_their_unwrapped_form() {
    let targets = [
        target(1, segment([-3., 0.1], [3., 0.3])),
        target(2, circle([0.5, 0.25], 1.25)),
    ];
    for (native, manual) in [cubic([0., 6.]), rational_quadratic()] {
        let native_report = certify(
            curve_source(native.clone(), SketchLineDomain::Segment),
            &targets,
        );
        let manual_report = certify(curve_source(manual, SketchLineDomain::Segment), &targets);
        assert!(native_report.ordered().len() >= 4, "{native_report:#?}");
        assert_same_incidences(&native_report, &manual_report);
        assert_eq!(native_report.source().curve, native);
    }
}
#[test]
fn a_full_period_source_is_closed_and_trims_cyclically() {
    let (native, _) = cubic([0., 6.]);
    let line = target(1, infinite([-3., 0.], [3., 0.]));
    let report = certify(curve_source(native, SketchLineDomain::Segment), &[line]);
    assert_eq!(report.ordered().len(), 2, "{report:#?}");
    let first = report
        .contact(report.ordered()[0].members()[0])
        .source_parameter_bounds();
    let pick = first.unwrap()[1] - 0.25;
    let selection = report
        .select_trim_at(SketchTrimPick::Parameter(pick), || false)
        .unwrap();
    assert_eq!(
        selection,
        SketchTrimSelection {
            before: Some(1),
            after: Some(0)
        }
    );
}
fn refusal(curve: SketchIncidenceCurve) -> SketchIncidenceCause {
    let line = source([[-3., 0.], [3., 0.]], SketchLineDomain::Segment);
    let options = SketchIncidenceOptions::v1(ERROR);
    let failure = certify_sketch_line_incidences(line, &[target(9, curve)], options, || false)
        .expect_err("refused periodic spline");
    assert_eq!(failure.completion(), SketchIncidenceCompletion::Invalid);
    assert_eq!(failure.target(), Some(9));
    failure.cause().clone()
}
#[test]
fn periodic_admission_names_every_refusal() {
    let knots = [0., 1., 2., 3.];
    let short = periodic(3, &HEXAGON[..2], &knots, &[1; 4], [0., 3.]);
    assert_eq!(
        refusal(SketchIncidenceCurve::PeriodicSpline(short)),
        SketchIncidenceCause::PeriodicShape {
            degree: 3,
            poles: 2,
            expected_poles: Some(3),
            multiplicities: vec![1; 4],
            weights: None,
        }
    );
    let uneven = periodic(3, &HEXAGON[..3], &knots, &[1, 1, 1, 2], [0., 3.]);
    let cause = refusal(SketchIncidenceCurve::PeriodicSpline(uneven));
    assert!(matches!(
        cause,
        SketchIncidenceCause::PeriodicShape {
            expected_poles: None,
            ..
        }
    ));
    let repeated = periodic(3, &HEXAGON[..3], &[0., 1., 1., 3.], &[1; 4], [0., 3.]);
    assert_eq!(
        refusal(SketchIncidenceCurve::PeriodicSpline(repeated)),
        SketchIncidenceCause::KnotSequence { index: 2 }
    );
    let outside = periodic(3, &HEXAGON[..3], &knots, &[1; 4], [-0.5, 2.]);
    assert!(matches!(
        refusal(SketchIncidenceCurve::PeriodicSpline(outside)),
        SketchIncidenceCause::SplineBounds { .. }
    ));
}
// 0.6 - (0.7 - 0.1) is exact but the binary64 period 0.7 - 0.1 is not, so the
// native shifted knots differ from the periodic curve's.
#[test]
fn an_inexact_period_shift_is_refused() {
    let knots = [0.1, 0.3, 0.45, 0.6, 0.7];
    let curve = periodic(2, &HEXAGON[..4], &knots, &[1; 5], [0.1, 0.7]);
    assert!(matches!(
        refusal(SketchIncidenceCurve::PeriodicSpline(curve)),
        SketchIncidenceCause::PeriodicKnotShift { .. }
    ));
}
#[test]
fn a_periodic_source_refuses_extension() {
    let (native, _) = cubic([0., 6.]);
    let options = SketchIncidenceOptions::v1(ERROR);
    let failure = certify_sketch_line_incidences(
        curve_source(native, SketchLineDomain::Support),
        &[],
        options,
        || false,
    )
    .expect_err("no spline extension rule");
    assert!(matches!(
        failure.cause(),
        SketchIncidenceCause::SourceDomain {
            source: "spline",
            ..
        }
    ));
}
