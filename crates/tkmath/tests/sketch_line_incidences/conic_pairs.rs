// SPDX-License-Identifier: GPL-2.0-only
//! Conic sources against conic targets: closed-form chart, reference-line,
//! exceptional-fibre, concentric and coincident cases.
use super::*;
use SketchContactTransition::{Crossing, Touching};
use SketchLinePosition::{AtEnd, Interior};
use SketchSweepChart::{Lower, Opposite, Reference, Upper};

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;
const SUPPORT: SketchLineDomain = SketchLineDomain::Support;

fn charts(report: &SketchLineIncidenceReport) -> Vec<SketchSweepChart> {
    ordered(report)
        .iter()
        .map(|contact| contact.source_sweep().unwrap().chart)
        .collect()
}
#[test]
fn circles_crossing_off_the_reference_line_have_rational_keys() {
    // x^2 + y^2 = 25 and (x - 8)^2 + y^2 = 25 meet at (4, 3) and (4, -3).
    let report = certify(
        curve_source(circle([0., 0.], 5.), SEGMENT),
        &[target(2, circle([8., 0.], 5.))],
    );
    let contacts = ordered(&report);
    assert_eq!(charts(&report), [Upper, Lower]);
    assert_point(contacts[0], [4., 3.]);
    assert_point(contacts[1], [4., -3.]);
    let key = contacts[0]
        .source_sweep()
        .unwrap()
        .cotangent_bounds
        .unwrap();
    assert!(key[0] <= -4. / 3. && -4. / 3. <= key[1]);
    for contact in &contacts {
        assert_eq!(
            summary(contact),
            (2, Interior, Interior, Crossing, [Some(1), Some(1)])
        );
    }
}
#[test]
fn internally_tangent_circles_touch_on_the_reference_ray() {
    // x^2 + y^2 = 9 and (x - 1)^2 + y^2 = 4 touch only at (3, 0).
    let report = certify(
        curve_source(circle([0., 0.], 3.), SEGMENT),
        &[target(2, circle([1., 0.], 2.))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [3., 0.]);
    assert_eq!(charts(&report), [Reference]);
    assert_eq!(
        summary(contacts[0]),
        (2, Interior, Interior, Touching, [Some(2), Some(2)])
    );
}
#[test]
fn both_points_on_one_center_line_form_an_exceptional_fibre() {
    // x^2 + y^2 = 9 and (x - 4)^2 + y^2 = 25 meet at (0, 3) and (0, -3): K = 0 twice.
    let report = certify(
        curve_source(circle([0., 0.], 3.), SEGMENT),
        &[target(2, circle([4., 0.], 5.))],
    );
    let contacts = ordered(&report);
    assert_eq!(charts(&report), [Upper, Lower]);
    assert_exact_point(contacts[0], [0., 3.]);
    assert_exact_point(contacts[1], [0., -3.]);
    for contact in &contacts {
        assert_eq!(
            contact.source_sweep().unwrap().cotangent_bounds,
            Some([0., 0.])
        );
        assert_eq!(summary(contact).3, Crossing);
    }
}
#[test]
fn exceptional_fibre_on_the_reference_line_is_read_in_the_second_chart() {
    // x^2 + y^2 = 9 and x^2 + (y - 4)^2 = 25 meet at (3, 0) and (-3, 0).
    let report = certify(
        curve_source(circle([0., 0.], 3.), SEGMENT),
        &[target(2, circle([0., 4.], 5.))],
    );
    let contacts = ordered(&report);
    assert_eq!(charts(&report), [Reference, Opposite]);
    assert_exact_point(contacts[0], [3., 0.]);
    assert_exact_point(contacts[1], [-3., 0.]);
}
#[test]
fn concentric_circle_and_ellipse_meet_symmetrically() {
    // x^2 + y^2 = 16 and x^2/25 + y^2/9 = 1: x^2 = 175/16, y^2 = 81/16.
    let target_ellipse = ellipse([0., 0.], [1., 0.], [5., 3.]);
    let report = certify(
        curve_source(circle([0., 0.], 4.), SEGMENT),
        &[target(2, target_ellipse)],
    );
    let contacts = ordered(&report);
    assert_eq!(charts(&report), [Upper, Upper, Lower, Lower]);
    let expected_x = 5. * 7f64.sqrt() / 4.;
    for contact in &contacts {
        let [x, y] = contact.point();
        assert!((x.abs() - expected_x).abs() < 1e-11, "{contact:?}");
        assert!((y.abs() - 2.25).abs() < 1e-11, "{contact:?}");
        assert_eq!(summary(contact).3, Crossing);
    }
}
#[test]
fn concentric_tangency_at_the_vertices_touches_on_the_reference_line() {
    // The circle of radius 5 touches the ellipse x^2/25 + y^2/9 = 1 at (5, 0) and (-5, 0).
    let report = certify(
        curve_source(circle([0., 0.], 5.), SEGMENT),
        &[target(2, ellipse([0., 0.], [1., 0.], [5., 3.]))],
    );
    let contacts = ordered(&report);
    assert_eq!(charts(&report), [Reference, Opposite]);
    assert_exact_point(contacts[0], [5., 0.]);
    assert_exact_point(contacts[1], [-5., 0.]);
    for contact in &contacts {
        assert_eq!(
            summary(contact),
            (2, Interior, Interior, Touching, [Some(2), Some(2)])
        );
    }
}
#[test]
fn ellipse_source_meets_a_circle_arc_with_sector_ends() {
    // x^2/4 + y^2 = 1 meets x^2 + y^2 = 1 at (0, 1) and (0, -1); the upper
    // quarter arc of the unit circle ends at (0, 1).
    let report = certify(
        curve_source(ellipse([0., 0.], [1., 0.], [2., 1.]), SEGMENT),
        &[target(2, arc([0., 0.], 1., [1., 0.], [0., 1.], true))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_point(contacts[0], [0., 1.]);
    assert_eq!(summary(contacts[0]).2, AtEnd);
    assert_eq!(contacts[0].multiplicity(), [Some(2), Some(2)]);
}
fn quarter(radius: f64) -> SketchIncidenceCurve {
    arc([0., 0.], radius, [1., 0.], [0., 1.], true)
}
#[test]
fn identical_conics_report_the_target_sweep_as_a_cyclic_overlap() {
    let next = arc([0., 0.], 1., [0., 1.], [-1., 0.], true);
    let report = certify(curve_source(quarter(1.), SEGMENT), &[target(3, next)]);
    let overlap = &report.targets()[0].overlaps()[0];
    assert_eq!(
        overlap.source_positions(),
        Some([AtEnd, SketchLinePosition::AfterEnd])
    );
    assert!(!overlap.opposite());
    assert!(report.select_trim([1., 1.], || false).is_ok());
    let middle = arc([0., 0.], 1., [1., 1.], [-1., 1.], true);
    let blocked = certify(curve_source(quarter(1.), SEGMENT), &[target(3, middle)]);
    assert_eq!(
        blocked.select_trim([1., 0.5], || false),
        Err(SketchSelectionError::Refused(
            SketchSelectionRefusal::Overlap { target_id: 3 }
        ))
    );
    let full = certify(
        curve_source(quarter(1.), SEGMENT),
        &[target(3, circle([0., 0.], 1.))],
    );
    assert_eq!(full.targets()[0].overlaps()[0].source_positions(), None);
    assert!(full.select_trim([1., 1.], || false).is_err());
}
#[test]
fn arc_extension_stops_before_a_coincident_arc_in_its_gap() {
    // x = -1/2 meets the unit circle at 120 and 240 degrees; the coincident
    // target arc covers 180 to 270 degrees.
    let targets = [
        target(1, segment([-0.5, -2.], [-0.5, 2.])),
        target(2, arc([0., 0.], 1., [-1., 0.], [0., -1.], true)),
    ];
    let report = certify(curve_source(quarter(1.), SUPPORT), &targets);
    let end = report
        .select_extension(SketchLineEnd::End, || false)
        .unwrap()
        .unwrap();
    let point = report.contact(report.ordered()[end].members()[0]).point();
    assert!(point[1] > 0., "{point:?}");
    assert_eq!(
        report.select_extension(SketchLineEnd::Start, || false),
        Err(SketchSelectionError::Refused(
            SketchSelectionRefusal::Overlap { target_id: 2 }
        ))
    );
}
#[test]
fn rotated_ellipses_cross_at_four_certified_points() {
    let source = curve_source(ellipse([0.25, -0.5], [3., 1.], [3., 1.5]), SEGMENT);
    let other = ellipse([0.5, 0.25], [-1., 2.], [2.5, 1.]);
    let report = certify(source, &[target(4, other)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 4, "{report:#?}");
    for contact in &contacts {
        assert!(contact.achieved_world_error() <= ERROR);
        assert_eq!(summary(contact).3, Crossing);
    }
}
