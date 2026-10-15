// SPDX-License-Identifier: GPL-2.0-only
//! B-spline targets: exact span pieces, knot contacts, overlaps, bounds and weights.
use super::*;
use SketchContactTransition::{Crossing, TargetEnd, Touching};
use SketchLinePosition::{AtEnd, AtStart, Interior};

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;

#[test]
fn closed_wrapped_spline_seam_and_self_crossing_form_one_exact_tie() {
    // Wrapped uniform cubic: both ends are (A + 4B + C)/6 = (0, 0), on the line.
    // The poles are symmetric about the origin, so the curve also crosses itself there.
    let poles = [
        [-6., 0.],
        [0., 0.],
        [6., 0.],
        [4., 3.],
        [-4., -3.],
        [-6., 0.],
        [0., 0.],
        [6., 0.],
    ];
    let knots: Vec<f64> = (0..12).map(f64::from).collect();
    let curve = spline(3, &poles, &knots, [3., 8.]);
    let report = certify(
        source([[-10., 0.], [10., 0.]], SEGMENT),
        &[target(5, curve)],
    );
    let seam = report
        .ordered()
        .iter()
        .find(|group| group.members().len() == 3)
        .unwrap_or_else(|| panic!("the seam contacts share one exact group: {report:#?}"));
    let ends: Vec<_> = seam
        .members()
        .iter()
        .map(|member| summary(report.contact(*member)))
        .collect();
    assert_eq!(
        ends,
        [
            (5, Interior, AtStart, TargetEnd, [None, Some(3)]),
            (5, Interior, Interior, Crossing, [Some(1), Some(1)]),
            (5, Interior, AtEnd, TargetEnd, [Some(3), None]),
        ]
    );
    assert_exact_point(report.contact(seam.members()[0]), [0., 0.]);
}

#[test]
fn inflection_of_a_cubic_is_a_triple_crossing() {
    // Bernstein y controls (-1, 1, -1, 1) expand to (2u - 1)^3; x = 3u.
    let poles = [[0., -1.], [1., 1.], [2., -1.], [3., 1.]];
    let curve = spline(3, &poles, &[0., 0., 0., 0., 1., 1., 1., 1.], [0., 1.]);
    let report = certify(source([[0., 0.], [3., 0.]], SEGMENT), &[target(5, curve)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [1.5, 0.]);
    assert_eq!(
        summary(contacts[0]),
        (5, Interior, Interior, Crossing, [Some(3), Some(3)])
    );
    assert_eq!(contacts[0].target_parameter_bounds(), Some([0.5, 0.5]));
}
#[test]
fn polyline_spline_crosses_each_span_once() {
    let poles = [[0., -1.], [1., 1.], [2., -1.], [3., 1.]];
    let curve = spline(1, &poles, &[0., 0., 1., 2., 3., 3.], [0., 3.]);
    let report = certify(source([[0., 0.], [4., 0.]], SEGMENT), &[target(5, curve)]);
    let points: Vec<[f64; 2]> = ordered(&report)
        .iter()
        .map(|contact| contact.point())
        .collect();
    assert_eq!(points, [[0.5, 0.], [1.5, 0.], [2.5, 0.]]);
}
#[test]
fn knot_vertex_touching_the_line_merges_both_spans() {
    let poles = [[0., -1.], [1., 1.], [2., -1.], [3., 1.]];
    let curve = spline(1, &poles, &[0., 0., 1., 2., 3., 3.], [0., 3.]);
    let report = certify(source([[0., 1.], [4., 1.]], SEGMENT), &[target(5, curve)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2);
    assert_exact_point(contacts[0], [1., 1.]);
    assert_eq!(
        summary(contacts[0]),
        (5, Interior, Interior, Touching, [Some(1), Some(1)])
    );
    assert_eq!(contacts[0].target_parameter_bounds(), Some([1., 1.]));
    assert_exact_point(contacts[1], [3., 1.]);
    assert_eq!(
        summary(contacts[1]),
        (5, Interior, AtEnd, TargetEnd, [Some(1), None])
    );
}
#[test]
fn knot_vertex_crossing_the_line_is_one_crossing() {
    let curve = spline(
        1,
        &[[0., -1.], [1., 0.], [2., 1.]],
        &[0., 0., 1., 2., 2.],
        [0., 2.],
    );
    let report = certify(source([[0., 0.], [4., 0.]], SEGMENT), &[target(5, curve)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [1., 0.]);
    assert_eq!(contacts[0].transition(), Crossing);
}
#[test]
fn collinear_span_is_an_overlap_without_boundary_contacts() {
    let poles = [[0., 1.], [1., 0.], [2., 0.], [3., 1.]];
    let curve = spline(1, &poles, &[0., 0., 1., 2., 3., 3.], [0., 3.]);
    let report = certify(source([[0., 0.], [4., 0.]], SEGMENT), &[target(5, curve)]);
    let incidences = &report.targets()[0];
    assert!(incidences.contacts().is_empty());
    assert_eq!(incidences.overlaps().len(), 1);
    assert_eq!(incidences.overlaps()[0].target_range(), Some([1., 2.]));
    assert_eq!(
        incidences.overlaps()[0].source_parameter_range_bounds(),
        Some([[0.25, 0.25], [0.5, 0.5]])
    );
}
#[test]
fn trimmed_bounds_drop_outside_roots_and_mark_exact_bound_contacts() {
    let poles = [[0., -1.], [1., 1.], [2., -1.], [3., 1.]];
    let curve = spline(1, &poles, &[0., 0., 1., 2., 3., 3.], [0.5, 2.25]);
    let report = certify(source([[0., 0.], [4., 0.]], SEGMENT), &[target(5, curve)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2);
    assert_eq!(
        summary(contacts[0]),
        (5, Interior, AtStart, TargetEnd, [None, Some(1)])
    );
    assert_eq!(
        summary(contacts[1]),
        (5, Interior, Interior, Crossing, [Some(1), Some(1)])
    );
}
#[test]
fn rational_quarter_circle_meets_the_diagonal_on_the_unit_circle() {
    // Weights (1, 1, 2) give x = (1 - u^2)/(1 + u^2), y = 2u/(1 + u^2).
    let quarter = SketchIncidenceCurve::Spline(SketchIncidenceSpline {
        degree: 2,
        poles: vec![[1., 0.], [1., 1.], [0., 1.]],
        weights: Some(vec![1., 1., 2.]),
        flat_knots: vec![0., 0., 0., 1., 1., 1.],
        bounds: [0., 1.],
    });
    let report = certify(source([[0., 0.], [2., 2.]], SEGMENT), &[target(5, quarter)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    let [x, y] = contacts[0].point();
    let slack = contacts[0].achieved_world_error() + 4. * f64::EPSILON;
    assert!((x.hypot(y) - 1.).abs() <= slack);
    assert!((x - y).abs() <= 2. * ERROR);
    // u = sqrt(2) - 1 solves u^2 + 2u - 1 = 0.
    let [lower, upper] = contacts[0].target_parameter_bounds().unwrap();
    let root = 2f64.sqrt() - 1.;
    assert!(lower <= root + f64::EPSILON && root - f64::EPSILON <= upper && upper - lower < 1e-11);
}
