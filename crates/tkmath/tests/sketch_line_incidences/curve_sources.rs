// SPDX-License-Identifier: GPL-2.0-only
//! Conic and spline sources against line, conic and spline targets (closed forms).
use super::*;
use SketchContactTransition::{Crossing, SourceEnd, Touching};
use SketchLinePosition::{AfterEnd, AtEnd, AtStart, Interior};
use SketchSweepChart::{Lower, Opposite, Reference, Upper};

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;
const SUPPORT: SketchLineDomain = SketchLineDomain::Support;

fn sweep(contact: &SketchLineContact) -> (SketchSweepChart, Option<[f64; 2]>) {
    let place = contact.source_sweep().expect("a conic-source contact");
    (place.chart, place.cotangent_bounds)
}
fn points(report: &SketchLineIncidenceReport) -> Vec<[f64; 2]> {
    ordered(report)
        .iter()
        .map(|contact| contact.point())
        .collect()
}
fn quarter_arc(radius: f64) -> SketchIncidenceCurve {
    arc([0., 0.], radius, [1., 0.], [0., 1.], true)
}
#[test]
fn full_circle_source_orders_contacts_by_sweep_from_its_axis() {
    // The x axis meets the unit circle at angle 0 (reference ray) and a half turn.
    let report = certify(
        curve_source(circle([0., 0.], 1.), SEGMENT),
        &[target(3, segment([-2., 0.], [2., 0.]))],
    );
    let contacts = ordered(&report);
    assert_eq!(points(&report), [[1., 0.], [-1., 0.]]);
    assert_eq!(sweep(contacts[0]), (Reference, None));
    assert_eq!(sweep(contacts[1]), (Opposite, None));
    for contact in &contacts {
        // Measured along the target line: it crosses the circle at both points.
        assert_eq!(
            summary(contact),
            (3, Interior, Interior, Crossing, [Some(1), Some(1)])
        );
        assert_eq!(contact.source_parameter_bounds(), None);
    }
    assert_eq!(contacts[0].target_parameter_bounds(), Some([0.75, 0.75]));
}
#[test]
fn quarter_arc_source_keeps_outside_contacts_beyond_its_end() {
    // x = 3 meets x^2 + y^2 = 25 at (3, 4) inside the quarter and (3, -4) outside.
    let line = target(4, segment([3., -10.], [3., 10.]));
    let report = certify(
        curve_source(quarter_arc(5.), SUPPORT),
        std::slice::from_ref(&line),
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2);
    assert_point(contacts[0], [3., 4.]);
    assert_point(contacts[1], [3., -4.]);
    // Cotangent keys -dot/det: -3/4 in the upper half, 3/4 in the lower half.
    let (chart, key) = sweep(contacts[0]);
    assert_eq!(chart, Upper);
    let [lower, upper] = key.unwrap();
    assert!(lower <= -0.75 && -0.75 <= upper);
    assert_eq!(sweep(contacts[1]).0, Lower);
    assert_eq!(contacts[0].source_position(), Interior);
    assert_eq!(contacts[1].source_position(), AfterEnd);
    let segment_report = certify(curve_source(quarter_arc(5.), SEGMENT), &[line]);
    assert_eq!(ordered(&segment_report).len(), 1);
}
#[test]
fn arc_ends_on_the_target_are_exact_start_and_end() {
    // y = 0 and x = 0 pass through both quarter-arc ends exactly.
    let targets = [
        target(1, segment([-9., 0.], [9., 0.])),
        target(2, segment([0., -9.], [0., 9.])),
    ];
    let report = certify(curve_source(quarter_arc(5.), SUPPORT), &targets);
    let positions: Vec<_> = ordered(&report)
        .iter()
        .map(|contact| {
            (
                contact.target_id(),
                contact.source_position(),
                sweep(contact).0,
            )
        })
        .collect();
    assert_eq!(
        positions,
        [
            (1, AtStart, Reference),
            (2, AtEnd, Upper),
            (1, AfterEnd, Opposite),
            (2, AfterEnd, Lower),
        ]
    );
    // The target parameter 7/9 is not dyadic: the point is certified, not exact.
    assert_point(ordered(&report)[1], [0., 5.]);
}
#[test]
fn clockwise_arc_source_sweeps_in_its_own_sense() {
    // Clockwise from (1, 0) to (0, -1): the lower quarter is inside.
    let arc_source = arc([0., 0.], 2., [1., 0.], [0., -1.], false);
    let report = certify(
        curve_source(arc_source, SEGMENT),
        &[target(4, segment([1., -5.], [1., 5.]))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert!(contacts[0].point()[1] < 0.);
    assert_eq!(sweep(contacts[0]).0, Upper);
}
#[test]
fn ellipse_source_places_its_axis_crossings_in_both_half_turns() {
    // x^2/4 + y^2 = 1 meets x = 0 at (0, 1) and (0, -1); both keys are exactly 0.
    let source = curve_source(ellipse([0., 0.], [2., 0.], [2., 1.]), SEGMENT);
    let report = certify(source, &[target(5, segment([0., -5.], [0., 5.]))]);
    let contacts = ordered(&report);
    assert_point(contacts[0], [0., 1.]);
    assert_point(contacts[1], [0., -1.]);
    assert_eq!(sweep(contacts[0]), (Upper, Some([0., 0.])));
    assert_eq!(sweep(contacts[1]), (Lower, Some([0., 0.])));
}
#[test]
fn circle_source_meets_a_polyline_spline_on_both_spans() {
    // Spans y = 0 (x from -2 to 0) and x = 0 (y from 0 to 2) meet the unit circle.
    let curve = spline(
        1,
        &[[-2., 0.], [0., 0.], [0., 2.]],
        &[0., 0., 1., 2., 2.],
        [0., 2.],
    );
    let report = certify(
        curve_source(circle([0., 0.], 1.), SEGMENT),
        &[target(6, curve)],
    );
    assert_eq!(points(&report), [[0., 1.], [-1., 0.]]);
    let contacts = ordered(&report);
    assert_eq!(contacts[0].target_parameter_bounds(), Some([1.5, 1.5]));
    assert_eq!(contacts[1].target_parameter_bounds(), Some([0.5, 0.5]));
}
fn polyline() -> SketchIncidenceCurve {
    spline(
        1,
        &[[-2., 0.], [2., 0.], [2., 2.]],
        &[0., 0., 1., 2., 2.],
        [0., 2.],
    )
}
#[test]
fn spline_source_orders_contacts_by_its_parameter() {
    // The first span x = -2 + 4s meets the unit circle at s = 1/4 and 3/4.
    let report = certify(
        curve_source(polyline(), SEGMENT),
        &[target(7, circle([0., 0.], 1.))],
    );
    let contacts = ordered(&report);
    assert_eq!(points(&report), [[-1., 0.], [1., 0.]]);
    assert_eq!(contacts[0].source_parameter_bounds(), Some([0.25, 0.25]));
    assert_eq!(
        summary(contacts[1]),
        (7, Interior, Interior, Crossing, [Some(1), Some(1)])
    );
}
#[test]
fn spline_source_knot_corner_touching_a_line_merges_its_spans() {
    // y = x - 2 passes the corner (2, 0); both spans stay on the same side.
    let report = certify(
        curve_source(polyline(), SEGMENT),
        &[target(8, infinite([2., 0.], [3., 1.]))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [2., 0.]);
    assert_eq!(
        summary(contacts[0]),
        (8, Interior, Interior, Touching, [Some(1), Some(1)])
    );
    assert_eq!(contacts[0].source_parameter_bounds(), Some([1., 1.]));
    assert_eq!(contacts[0].target_parameter_bounds(), Some([0., 0.]));
}
#[test]
fn spline_source_end_on_a_target_is_a_one_sided_source_end() {
    let report = certify(
        curve_source(polyline(), SEGMENT),
        &[target(9, segment([-2., -1.], [-2., 1.]))],
    );
    let contacts = ordered(&report);
    assert_eq!(
        summary(contacts[0]),
        (9, AtStart, Interior, SourceEnd, [None, Some(1)])
    );
    assert_eq!(contacts[0].target_parameter_bounds(), Some([0.5, 0.5]));
}
#[test]
fn spline_source_ignores_line_target_points_beyond_its_segment() {
    let report = certify(
        curve_source(polyline(), SEGMENT),
        &[target(9, segment([0., 1.], [0., 3.]))],
    );
    assert!(report.ordered().is_empty());
}
#[test]
fn spline_support_domain_is_refused_by_name() {
    let failure = certify_sketch_line_incidences(
        curve_source(polyline(), SUPPORT),
        &[],
        SketchIncidenceOptions::v1(ERROR),
        || false,
    )
    .unwrap_err();
    assert_eq!(
        failure.cause(),
        &SketchIncidenceCause::SourceDomain {
            source: "spline",
            domain: SUPPORT
        }
    );
    assert_eq!(failure.completion(), SketchIncidenceCompletion::Invalid);
}
#[test]
fn unbounded_line_source_is_refused_by_name() {
    let source = curve_source(infinite([0., 0.], [1., 0.]), SEGMENT);
    let failure =
        certify_sketch_line_incidences(source, &[], SketchIncidenceOptions::v1(ERROR), || false)
            .unwrap_err();
    assert_eq!(failure.cause(), &SketchIncidenceCause::UnboundedSource);
}
#[test]
fn spline_on_a_line_target_is_a_named_uncertified_overlap() {
    let failure = certify_sketch_line_incidences(
        curve_source(polyline(), SEGMENT),
        &[target(9, segment([0., 0.], [1., 0.]))],
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
    assert_eq!(failure.target(), Some(9));
    assert_eq!(failure.completion(), SketchIncidenceCompletion::Incomplete);
}
