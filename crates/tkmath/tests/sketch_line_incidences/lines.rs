// SPDX-License-Identifier: GPL-2.0-only
//! Segment and supporting-line targets, including parallel and collinear cases.
use super::*;
use SketchContactTransition::{Crossing, TargetEnd};
use SketchLinePosition::{AfterEnd, AtStart, BeforeStart, Interior};

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;
const SUPPORT: SketchLineDomain = SketchLineDomain::Support;

#[test]
fn crossing_segments_meet_at_exact_parameters_on_both_lines() {
    let report = certify(
        source([[0., 0.], [4., 0.]], SEGMENT),
        &[target(9, segment([1., -1.], [1., 1.]))],
    );
    let contacts = ordered(&report);
    assert_eq!(contacts.len(), 1);
    assert_exact_point(contacts[0], [1., 0.]);
    assert_eq!(
        summary(contacts[0]),
        (9, Interior, Interior, Crossing, [Some(1), Some(1)])
    );
    assert_eq!(contacts[0].target_parameter_bounds(), Some([0.5, 0.5]));
    assert_eq!(contacts[0].source_parameter_bounds(), Some([0.25, 0.25]));
}
#[test]
fn target_endpoint_on_the_line_is_a_one_sided_contact() {
    let report = certify(
        source([[0., 0.], [4., 0.]], SEGMENT),
        &[target(9, segment([2., 0.], [2., 3.]))],
    );
    let contacts = ordered(&report);
    assert_exact_point(contacts[0], [2., 0.]);
    assert_eq!(
        summary(contacts[0]),
        (9, Interior, AtStart, TargetEnd, [None, Some(1)])
    );
}
#[test]
fn non_dyadic_line_parameter_constructs_the_exact_point_within_its_bound() {
    // x = 3t meets x = 1 at t = 1/3.
    let report = certify(
        source([[0., 0.], [3., 0.]], SEGMENT),
        &[target(9, segment([1., -1.], [1., 1.]))],
    );
    let contacts = ordered(&report);
    assert_point(contacts[0], [1., 0.]);
    let [lower, upper] = contacts[0].source_parameter_bounds().unwrap();
    assert!(lower < upper && lower <= 1. / 3. && 1. / 3. <= upper);
}
#[test]
fn bounded_target_short_of_the_line_has_no_contact_but_its_support_does() {
    let short = segment([1., 1.], [1., 3.]);
    let report = certify(source([[0., 0.], [4., 0.]], SUPPORT), &[target(9, short)]);
    assert!(report.targets()[0].contacts().is_empty());
    let report = certify(
        source([[0., 0.], [4., 0.]], SUPPORT),
        &[target(9, infinite([1., 1.], [1., 3.]))],
    );
    let contacts = ordered(&report);
    assert_exact_point(contacts[0], [1., 0.]);
    assert_eq!(contacts[0].target_parameter_bounds(), Some([-0.5, -0.5]));
    assert_eq!(contacts[0].target_position(), Interior);
}
#[test]
fn segment_domain_retains_but_does_not_order_support_contacts() {
    let beyond = [target(9, segment([3., -1.], [3., 1.]))];
    let support = certify(source([[0., 0.], [1., 0.]], SUPPORT), &beyond);
    assert_eq!(ordered(&support)[0].source_position(), AfterEnd);
    let bounded = certify(source([[0., 0.], [1., 0.]], SEGMENT), &beyond);
    assert!(bounded.ordered().is_empty());
    assert_eq!(
        bounded.targets()[0].contacts()[0].source_position(),
        AfterEnd
    );
}
#[test]
fn parallel_distinct_lines_complete_empty() {
    let report = certify(
        source([[0., 0.], [4., 0.]], SUPPORT),
        &[target(9, infinite([0., 1.], [4., 1.]))],
    );
    assert!(report.targets()[0].contacts().is_empty());
    assert!(report.targets()[0].overlaps().is_empty());
}
#[test]
fn collinear_segments_report_an_exact_oriented_overlap() {
    let report = certify(
        source([[0., 0.], [4., 0.]], SEGMENT),
        &[target(9, segment([1., 0.], [3., 0.]))],
    );
    let overlap = &report.targets()[0].overlaps()[0];
    assert!(report.targets()[0].contacts().is_empty());
    assert_eq!(overlap.target_range(), Some([0., 1.]));
    assert_eq!(
        overlap.source_parameter_range_bounds(),
        Some([[0.25, 0.25], [0.75, 0.75]])
    );
    assert_eq!(overlap.source_positions(), Some([Interior, Interior]));
    assert!(!overlap.opposite());
    let report = certify(
        source([[0., 0.], [4., 0.]], SEGMENT),
        &[target(9, segment([5., 0.], [-1., 0.]))],
    );
    let overlap = &report.targets()[0].overlaps()[0];
    assert_eq!(overlap.source_positions(), Some([AfterEnd, BeforeStart]));
    assert!(overlap.opposite());
}
#[test]
fn collinear_unbounded_line_overlaps_everywhere() {
    let report = certify(
        source([[0., 0.], [4., 0.]], SUPPORT),
        &[target(9, infinite([7., 0.], [-1., 0.]))],
    );
    let overlap = &report.targets()[0].overlaps()[0];
    assert_eq!(overlap.target_range(), None);
    assert_eq!(overlap.source_parameter_range_bounds(), None);
    assert!(overlap.opposite());
}
