// SPDX-License-Identifier: GPL-2.0-only
//! Exact total order across targets, with ties proved rather than approximated.
use super::*;

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;

fn group_sizes(report: &SketchLineIncidenceReport) -> Vec<usize> {
    report
        .ordered()
        .iter()
        .map(|group| group.members().len())
        .collect()
}
#[test]
fn circles_touching_on_the_line_share_one_dyadic_group() {
    let targets = [
        target(1, circle([0., 0.], 1.)),
        target(2, circle([2., 0.], 1.)),
    ];
    let report = certify(source([[-2., 0.], [4., 0.]], SEGMENT), &targets);
    assert_eq!(group_sizes(&report), [1, 2, 1]);
    assert_eq!(report.ordered()[1].members(), [[0, 1], [1, 0]]);
}
#[test]
fn non_dyadic_tie_between_a_line_and_a_circle_is_proved_by_a_common_factor() {
    // x = 3t meets x = 1 and (x - 2)^2 = 1 at t = 1/3; the circle also at t = 1.
    let targets = [
        target(1, segment([1., -1.], [1., 1.])),
        target(2, circle([2., 0.], 1.)),
    ];
    let report = certify(source([[0., 0.], [3., 0.]], SEGMENT), &targets);
    assert_eq!(group_sizes(&report), [2, 1]);
    let tied = report.ordered()[0].members();
    assert_eq!(tied, [[0, 0], [1, 0]]);
    assert_point(report.contact(tied[1]), [1., 0.]);
}
#[test]
fn irrational_tie_between_two_circles_and_a_rational_spline_is_exact() {
    // On x = 1/2 both circles and the unit quarter arc pass through (1/2, sqrt(3)/2).
    let quarter = SketchIncidenceCurve::Spline(SketchIncidenceSpline {
        degree: 2,
        poles: vec![[1., 0.], [1., 1.], [0., 1.]],
        weights: Some(vec![1., 1., 2.]),
        flat_knots: vec![0., 0., 0., 1., 1., 1.],
        bounds: [0., 1.],
    });
    let targets = [
        target(1, circle([0., 0.], 1.)),
        target(2, circle([1., 0.], 1.)),
        target(3, quarter),
    ];
    let report = certify(source([[0.5, -2.], [0.5, 2.]], SEGMENT), &targets);
    assert_eq!(group_sizes(&report), [2, 3]);
    let upper = &report.ordered()[1];
    let ids: Vec<u64> = upper
        .members()
        .iter()
        .map(|member| report.contact(*member).target_id())
        .collect();
    assert_eq!(ids, [1, 2, 3]);
}
#[test]
fn adjacent_radius_circles_are_strictly_ordered() {
    let radius = 1f64;
    let larger = f64::from_bits(radius.to_bits() + 1);
    let targets = [
        target(1, circle([0., 0.], larger)),
        target(2, circle([0., 0.], radius)),
    ];
    let report = certify(source([[0., -0.5], [3., -0.5]], SEGMENT), &targets);
    assert_eq!(group_sizes(&report), [1, 1]);
    let ids: Vec<u64> = ordered(&report)
        .iter()
        .map(|contact| contact.target_id())
        .collect();
    assert_eq!(ids, [2, 1], "the smaller circle is met first going outward");
}
#[test]
fn order_is_identical_for_reordered_targets_up_to_member_indices() {
    let first = [
        target(1, circle([0., 0.], 1.)),
        target(2, segment([0.5, -3.], [0.5, 3.])),
    ];
    let second = [first[1].clone(), first[0].clone()];
    let line = source([[-2., 0.25], [2., 0.25]], SEGMENT);
    let ids = |report: &SketchLineIncidenceReport| -> Vec<u64> {
        ordered(report)
            .iter()
            .map(|contact| contact.target_id())
            .collect()
    };
    assert_eq!(
        ids(&certify(line.clone(), &first)),
        ids(&certify(line, &second))
    );
}
