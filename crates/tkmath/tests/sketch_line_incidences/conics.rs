// SPDX-License-Identifier: GPL-2.0-only
//! Circles, arcs and ellipses with closed-form intersections.
use super::*;
use SketchContactTransition::{Crossing, TargetEnd, Touching};
use SketchLinePosition::{AtEnd, AtStart, Interior};

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;

#[test]
fn line_through_unit_circle_crosses_at_exact_dyadic_points() {
    // x = -2 + 4t meets x^2 = 1 at t = 1/4 and 3/4.
    let report = certify(
        source([[-2., 0.], [2., 0.]], SEGMENT),
        &[target(7, circle([0., 0.], 1.))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2);
    assert_exact_point(contacts[0], [-1., 0.]);
    assert_exact_point(contacts[1], [1., 0.]);
    for contact in &contacts {
        assert_eq!(
            summary(contact),
            (7, Interior, Interior, Crossing, [Some(1), Some(1)])
        );
        assert_eq!(contact.target_parameter_bounds(), None);
    }
    assert_eq!(contacts[0].source_parameter_bounds(), Some([0.25, 0.25]));
    assert_eq!(report.max_achieved_world_error(), 0.);
}
#[test]
fn tangent_line_touches_with_one_double_root() {
    let report = certify(
        source([[-2., 1.], [2., 1.]], SEGMENT),
        &[target(7, circle([0., 0.], 1.))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [0., 1.]);
    assert_eq!(
        summary(contacts[0]),
        (7, Interior, Interior, Touching, [Some(2), Some(2)])
    );
}
#[test]
fn missing_line_is_a_completed_empty_result() {
    let report = certify(
        source([[-2., 1.5], [2., 1.5]], SEGMENT),
        &[target(7, circle([0., 0.], 1.))],
    );
    assert!(report.ordered().is_empty());
    assert!(report.targets()[0].contacts().is_empty());
}
#[test]
fn pythagorean_chord_has_rational_points_between_dyadic_parameters() {
    // y = -10 + 20t meets 9 + y^2 = 25 at t = 3/10 and 7/10.
    let report = certify(
        source([[3., -10.], [3., 10.]], SEGMENT),
        &[target(7, circle([0., 0.], 5.))],
    );
    let contacts = ordered(&report);
    assert_point(contacts[0], [3., -4.]);
    assert_point(contacts[1], [3., 4.]);
    let [lower, upper] = contacts[0].source_parameter_bounds().unwrap();
    assert!(lower < upper && lower <= 0.3 && 0.3 <= upper);
}
#[test]
fn irrational_chord_is_enclosed_with_a_certified_error() {
    let report = certify(
        source([[-2., 0.5], [2., 0.5]], SEGMENT),
        &[target(7, circle([0., 0.], 1.))],
    );
    let contacts = ordered(&report);
    let half_root_three = 3f64.sqrt() / 2.;
    for (contact, sign) in contacts.iter().zip([-1., 1.]) {
        assert_eq!(contact.point()[1], 0.5);
        // The binary64 square root is within one rounding of the exact value.
        let offset = (contact.point()[0] - sign * half_root_three).abs();
        assert!(
            offset <= contact.achieved_world_error() + f64::EPSILON,
            "{contact:?}"
        );
        assert!(0. < contact.achieved_world_error() && contact.achieved_world_error() <= ERROR);
        let [lower, upper] = contact.world_bounds()[0];
        assert!(lower < upper);
    }
}
#[test]
fn counterclockwise_semicircle_keeps_only_its_upper_contact() {
    let upper = arc([0., 0.], 1., [2., 0.], [-3., 0.], true);
    let report = certify(source([[0., -2.], [0., 2.]], SEGMENT), &[target(3, upper)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [0., 1.]);
    assert_eq!(
        summary(contacts[0]),
        (3, Interior, Interior, Crossing, [Some(1), Some(1)])
    );
}
#[test]
fn clockwise_semicircle_keeps_only_its_lower_contact() {
    let lower = arc([0., 0.], 1., [1., 0.], [-1., 0.], false);
    let report = certify(source([[0., -2.], [0., 2.]], SEGMENT), &[target(3, lower)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [0., -1.]);
}
#[test]
fn arc_endpoints_on_the_line_are_exact_one_sided_target_ends() {
    let upper = arc([0., 0.], 1., [1., 0.], [-1., 0.], true);
    let report = certify(source([[-2., 0.], [2., 0.]], SEGMENT), &[target(3, upper)]);
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2);
    assert_exact_point(contacts[0], [-1., 0.]);
    assert_eq!(
        summary(contacts[0]),
        (3, Interior, AtEnd, TargetEnd, [Some(1), Some(1)])
    );
    assert_eq!(
        summary(contacts[1]),
        (3, Interior, AtStart, TargetEnd, [Some(1), Some(1)])
    );
    let clockwise = arc([0., 0.], 1., [-1., 0.], [1., 0.], false);
    let report = certify(
        source([[-2., 0.], [2., 0.]], SEGMENT),
        &[target(3, clockwise)],
    );
    let positions: Vec<_> = ordered(&report)
        .iter()
        .map(|contact| contact.target_position())
        .collect();
    assert_eq!(positions, [AtStart, AtEnd]);
}
#[test]
fn reflex_arc_includes_points_beyond_a_half_turn() {
    // Counterclockwise from +x to -y sweeps three quarters of a turn.
    let reflex = arc([0., 0.], 1., [1., 0.], [0., -1.], true);
    let report = certify(
        source([[0., -2.], [0., 2.]], SEGMENT),
        &[target(3, reflex.clone())],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2);
    assert_eq!(contacts[0].target_position(), AtEnd);
    assert_eq!(contacts[1].target_position(), Interior);
    let report = certify(
        source([[-2., -0.5], [2., -0.5]], SEGMENT),
        &[target(3, reflex)],
    );
    let contacts = ordered(&report);
    assert_eq!(
        contacts.len(),
        1,
        "only the third-quadrant point lies in the sweep"
    );
    assert!(contacts[0].point()[0] < 0.);
}
#[test]
fn ellipse_axes_and_tangent_follow_the_implicit_form() {
    let horizontal = ellipse([1., 1.], [5., 0.], [2., 1.]);
    let report = certify(
        source([[1., -5.], [1., 5.]], SEGMENT),
        &[target(4, horizontal.clone())],
    );
    let contacts = ordered(&report);
    assert_exact_point(contacts[0], [1., 0.]);
    assert_point(contacts[1], [1., 2.]);
    let vertical = ellipse([1., 1.], [0., 3.], [2., 1.]);
    let report = certify(
        source([[1., -5.], [1., 5.]], SEGMENT),
        &[target(4, vertical)],
    );
    let contacts = ordered(&report);
    assert_point(contacts[0], [1., -1.]);
    assert_point(contacts[1], [1., 3.]);
    let report = certify(
        source([[-3., 2.], [5., 2.]], SEGMENT),
        &[target(4, horizontal)],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [1., 2.]);
    assert_eq!(contacts[0].transition(), Touching);
}
#[test]
fn rotated_ellipse_meets_its_major_axis_at_the_major_radius() {
    let rotated = ellipse([0., 0.], [1., 1.], [2., 1.]);
    let report = certify(
        source([[-4., -4.], [4., 4.]], SEGMENT),
        &[target(4, rotated)],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 2);
    for contact in contacts {
        let radius = contact.point()[0].hypot(contact.point()[1]);
        let slack = contact.achieved_world_error() + 4. * f64::EPSILON;
        assert!((radius - 2.).abs() <= slack, "{contact:?}");
        assert_eq!(
            contact.point()[0],
            contact.point()[1],
            "the major axis is the diagonal"
        );
    }
}
