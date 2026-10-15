// SPDX-License-Identifier: GPL-2.0-only
//! Spline sources against spline targets (closed forms).
use super::*;
use SketchContactTransition::{Crossing, Touching};
use SketchLinePosition::Interior;

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;

fn polyline(points: &[[f64; 2]]) -> SketchIncidenceCurve {
    let mut knots = vec![0.];
    knots.extend((0..points.len()).map(|index| index as f64));
    knots.push((points.len() - 1) as f64);
    spline(1, points, &knots, [0., (points.len() - 1) as f64])
}
fn bezier(points: &[[f64; 2]]) -> SketchIncidenceCurve {
    let degree = points.len() - 1;
    let mut knots = vec![0.; degree + 1];
    knots.extend(vec![1.; degree + 1]);
    spline(degree, points, &knots, [0., 1.])
}
fn source_of(curve: SketchIncidenceCurve) -> SketchIncidenceSource {
    curve_source(curve, SEGMENT)
}
#[test]
fn polylines_cross_on_their_spans_with_exact_parameters() {
    let source = source_of(polyline(&[[-2., 0.], [2., 0.], [2., 2.]]));
    let other = polyline(&[[0., -1.], [0., 1.], [3., 1.]]);
    let report = certify(source, &[target(1, other)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2);
    assert_exact_point(contacts[0], [0., 0.]);
    assert_eq!(contacts[0].source_parameter_bounds(), Some([0.5, 0.5]));
    assert_eq!(contacts[0].target_parameter_bounds(), Some([0.5, 0.5]));
    assert_point(contacts[1], [2., 1.]);
    let [lower, upper] = contacts[1].target_parameter_bounds().unwrap();
    assert!(lower <= 5. / 3. && 5. / 3. <= upper);
    for contact in &contacts {
        assert_eq!(
            summary(contact),
            (1, Interior, Interior, Crossing, [Some(1), Some(1)])
        );
    }
}
#[test]
fn line_spline_touches_a_parabola_at_its_vertex() {
    // y = 1 meets y = 2x - x^2 (x = 2u, y = 4u(1-u)) doubly at (1, 1).
    let source = source_of(polyline(&[[0., 1.], [2., 1.]]));
    let report = certify(
        source,
        &[target(2, bezier(&[[0., 0.], [1., 2.], [2., 0.]]))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [1., 1.]);
    assert_eq!(
        summary(contacts[0]),
        (2, Interior, Interior, Touching, [Some(2), Some(2)])
    );
}
#[test]
fn curved_spans_cross_at_irrational_points() {
    // (2s, 1 - 4s + 4s^2) meets y = 2x - x^2 where 8s^2 - 8s + 1 = 0.
    let source = source_of(bezier(&[[0., 1.], [1., -1.], [2., 1.]]));
    let report = certify(
        source,
        &[target(2, bezier(&[[0., 0.], [1., 2.], [2., 0.]]))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2);
    let offset = 2f64.sqrt() / 4.;
    for (contact, s) in contacts.iter().zip([0.5 - offset, 0.5 + offset]) {
        let [lower, upper] = contact.source_parameter_bounds().unwrap();
        assert!(lower - 1e-15 <= s && s <= upper + 1e-15, "{contact:?}");
        // On the parabola u = s, so both parameters enclose the same value.
        let [lower, upper] = contact.target_parameter_bounds().unwrap();
        assert!(lower - 1e-15 <= s && s <= upper + 1e-15, "{contact:?}");
        assert_eq!(summary(contact).3, Crossing);
    }
}
#[test]
fn target_corner_crossing_and_touching_follow_the_half_tangents() {
    let source = || source_of(polyline(&[[-1., 0.], [3., 0.]]));
    let through = certify(
        source(),
        &[target(3, polyline(&[[0., -1.], [1., 0.], [0., 1.]]))],
    );
    let contacts = ordered(&through);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [1., 0.]);
    assert_eq!(
        summary(contacts[0]),
        (3, Interior, Interior, Crossing, [Some(1), Some(1)])
    );
    assert_eq!(contacts[0].target_parameter_bounds(), Some([1., 1.]));
    let bounce = certify(
        source(),
        &[target(3, polyline(&[[0., 1.], [1., 0.], [2., 1.]]))],
    );
    assert_eq!(summary(ordered(&bounce)[0]).3, Touching);
}
#[test]
fn source_corner_touching_a_straight_target_merges_source_spans() {
    let source = source_of(polyline(&[[-1., 1.], [0., 0.], [1., 1.]]));
    let report = certify(
        source,
        &[target(4, bezier(&[[-1., 0.], [0., 0.], [1., 0.]]))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [0., 0.]);
    assert_eq!(
        summary(contacts[0]),
        (4, Interior, Interior, Touching, [Some(1), Some(1)])
    );
    assert_eq!(contacts[0].source_parameter_bounds(), Some([1., 1.]));
}
#[test]
fn unevenly_parameterized_straight_target_span_is_a_named_exceptional_fibre() {
    let source = source_of(polyline(&[[0., 0.], [3., 0.]]));
    let straight = bezier(&[[1.5, -2.], [1.5, 1.], [1.5, 2.]]);
    let failure = certify_sketch_line_incidences(
        source,
        &[target(5, straight)],
        SketchIncidenceOptions::v1(ERROR),
        || false,
    )
    .unwrap_err();
    assert_eq!(
        failure.cause(),
        &SketchIncidenceCause::ExceptionalFiber {
            range: Some([0., 1.])
        }
    );
    assert_eq!(failure.completion(), SketchIncidenceCompletion::Incomplete);
}
#[test]
fn coincident_spline_spans_are_a_named_uncertified_overlap() {
    let source = source_of(polyline(&[[0., 0.], [3., 0.]]));
    let failure = certify_sketch_line_incidences(
        source,
        &[target(6, polyline(&[[1., 0.], [2., 0.]]))],
        SketchIncidenceOptions::v1(ERROR),
        || false,
    )
    .unwrap_err();
    assert_eq!(
        failure.cause(),
        &SketchIncidenceCause::UncertifiedOverlap {
            range: Some([0., 1.])
        }
    );
}
