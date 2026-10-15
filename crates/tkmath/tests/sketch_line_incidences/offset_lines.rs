// SPDX-License-Identifier: GPL-2.0-only
//! Line offsets (ADR 0080): the offset of `A -> B` by `d` is the parallel line
//! `d` to the right of the travel direction, parameterized like its basis.
//! Expected points are derived by hand below.
use super::offset_circles::offset;
use super::*;

/// The offset of the segment `(-2, -1) -> (2, -1)` by -1: the segment y = 0, x in [-2, 2].
fn lifted_source(domain: SketchLineDomain) -> SketchIncidenceSource {
    curve_source(offset(segment([-2., -1.], [2., -1.]), -1.), domain)
}
/// The offset of the upward segment x = -1 by 0.5: x = -0.5, y in [-3, 3].
fn shifted_target() -> SketchIncidenceCurve {
    offset(segment([-1., -3.], [-1., 3.]), 0.5)
}
#[test]
fn a_line_meets_a_line_offset_at_its_exact_point() {
    let report = certify(
        source([[-2., 0.], [2., 0.]], SketchLineDomain::Segment),
        &[target(3, shifted_target())],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1, "{report:#?}");
    assert_exact_point(contacts[0], [-0.5, 0.]);
    assert_eq!(contacts[0].source_parameter_bounds(), Some([0.375, 0.375]));
    assert_eq!(contacts[0].target_parameter_bounds(), Some([0.5, 0.5]));
    assert_eq!(contacts[0].transition(), SketchContactTransition::Crossing);
}
// The offset of (0,0) -> (1,1) by 1 is y = x - sqrt(2), crossing y = 0 at sqrt(2).
#[test]
fn an_irrational_line_offset_is_certified() {
    let report = certify(
        source([[-3., 0.], [3., 0.]], SketchLineDomain::Segment),
        &[target(4, offset(infinite([0., 0.], [1., 1.]), 1.))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1, "{report:#?}");
    assert_point(contacts[0], [2f64.sqrt(), 0.]);
}
// The offset of (0,0) -> (3,4) by 5 moves by (4,-3): it meets y = 0 at x = 6.25.
#[test]
fn a_pythagorean_line_offset_meets_lines_and_circles_exactly() {
    let shifted = offset(segment([0., 0.], [3., 4.]), 5.);
    let line = certify(
        source([[0., 0.], [8., 0.]], SketchLineDomain::Segment),
        &[target(5, shifted.clone())],
    );
    assert_exact_point(ordered(&line)[0], [6.25, 0.]);
    let circle_source = curve_source(circle([6.25, 0.], 1.), SketchLineDomain::Segment);
    let around = certify(circle_source, &[target(5, shifted)]);
    let contacts = ordered(&around);
    assert_eq!(contacts.len(), 2, "{around:#?}");
    assert_point(contacts[0], [6.85, 0.8]);
    assert_point(contacts[1], [5.65, -0.8]);
}
#[test]
fn a_line_offset_source_meets_circles_lines_and_offsets() {
    let targets = [
        target(1, circle([0., 0.], 1.)),
        target(2, shifted_target()),
        target(3, segment([0.5, -1.], [0.5, 1.])),
    ];
    let report = certify(lifted_source(SketchLineDomain::Segment), &targets);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 4, "{report:#?}");
    let expected = [[-1., 0.], [-0.5, 0.], [0.5, 0.], [1., 0.]];
    for (contact, point) in contacts.iter().zip(expected) {
        assert_exact_point(contact, point);
        let t = (point[0] + 2.) / 4.;
        assert_eq!(contact.source_parameter_bounds(), Some([t, t]));
    }
}
// y = x - sqrt(2) is tangent to the unit circle at (sqrt(2)/2, -sqrt(2)/2).
#[test]
fn an_irrational_line_offset_source_touches_a_circle() {
    let tangent = curve_source(
        offset(segment([0., 0.], [1., 1.]), 1.),
        SketchLineDomain::Support,
    );
    let report = certify(tangent, &[target(1, circle([0., 0.], 1.))]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1, "{report:#?}");
    assert_eq!(contacts[0].transition(), SketchContactTransition::Touching);
    assert_eq!(contacts[0].multiplicity(), [Some(2), Some(2)]);
    let half = 2f64.sqrt() / 2.;
    assert_point(contacts[0], [half, -half]);
}
// The lifted source is the segment y = 0 parameterized as (-2,0) -> (2,0).
#[test]
fn a_line_offset_source_reports_like_its_exact_line() {
    let targets = [
        target(
            1,
            spline(
                3,
                &[[-2., -1.], [-1., 2.], [0., -2.], [1., 2.], [2., -1.]],
                &[0., 0., 0., 0., 0.5, 1., 1., 1., 1.],
                [0., 1.],
            ),
        ),
        target(2, ellipse([0.25, 0.5], [2., 1.], [1.5, 0.75])),
        target(3, segment([-1.5, -0.5], [1.75, 0.25])),
    ];
    let lifted = certify(lifted_source(SketchLineDomain::Support), &targets);
    let line = certify(
        source([[-2., 0.], [2., 0.]], SketchLineDomain::Support),
        &targets,
    );
    let (lifted, line) = (ordered(&lifted), ordered(&line));
    assert!(lifted.len() >= 6 && lifted.len() == line.len());
    for (offset_contact, line_contact) in lifted.iter().zip(&line) {
        assert_eq!(summary(offset_contact), summary(line_contact));
        let slack = offset_contact.achieved_world_error() + line_contact.achieved_world_error();
        for axis in 0..2 {
            let gap = (offset_contact.point()[axis] - line_contact.point()[axis]).abs();
            assert!(gap <= slack, "{offset_contact:?} vs {line_contact:?}");
        }
    }
}
#[test]
fn a_line_offset_source_certifies_a_coincident_segment() {
    let report = certify(
        lifted_source(SketchLineDomain::Segment),
        &[target(6, segment([-1., 0.], [3., 0.]))],
    );
    let overlap = &report.targets()[0].overlaps()[0];
    assert_eq!(
        overlap.source_parameter_range_bounds(),
        Some([[0.25, 0.25], [1.25, 1.25]])
    );
    assert_eq!(
        overlap.source_positions(),
        Some([SketchLinePosition::Interior, SketchLinePosition::AfterEnd])
    );
}
#[test]
fn a_line_offset_source_trims_by_its_projected_pick() {
    let targets = [target(1, circle([0., 0.], 1.)), target(2, shifted_target())];
    let report = certify(lifted_source(SketchLineDomain::Segment), &targets);
    let around = report.select_trim([-0.75, 0.3], || false).unwrap();
    assert_eq!(
        around,
        SketchTrimSelection {
            before: Some(0),
            after: Some(1)
        }
    );
}
