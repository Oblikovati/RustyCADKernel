// SPDX-License-Identifier: GPL-2.0-only
//! Trim and extension of conic and spline sources, including cyclic trims.
use super::*;

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;
const SUPPORT: SketchLineDomain = SketchLineDomain::Support;

fn trim(before: Option<usize>, after: Option<usize>) -> SketchTrimSelection {
    SketchTrimSelection { before, after }
}
fn refused(refusal: SketchSelectionRefusal) -> Result<SketchTrimSelection, SketchSelectionError> {
    Err(SketchSelectionError::Refused(refusal))
}
fn axis_circle_report() -> SketchLineIncidenceReport {
    certify(
        curve_source(circle([0., 0.], 1.), SEGMENT),
        &[target(1, segment([-2., 0.], [2., 0.]))],
    )
}
#[test]
fn full_circle_trim_opens_between_the_cyclic_neighbors_of_a_central_pick() {
    let report = axis_circle_report();
    // Above the center: between the reference crossing (1, 0) and the opposite one.
    assert_eq!(
        report.select_trim([0., 5.], || false),
        Ok(trim(Some(0), Some(1)))
    );
    // Below the center the removed part runs from (-1, 0) across the start to (1, 0).
    assert_eq!(
        report.select_trim([0., -5.], || false),
        Ok(trim(Some(1), Some(0)))
    );
    assert_eq!(
        report.select_trim([3., 0.], || false),
        refused(SketchSelectionRefusal::PickAtIncidence { group: 0 })
    );
    assert_eq!(
        report.select_trim([0., 0.], || false),
        refused(SketchSelectionRefusal::PickOutsideSegment {
            pick: SketchTrimPick::Point([0., 0.])
        })
    );
}
#[test]
fn full_circle_with_one_cut_point_is_not_opened() {
    let report = certify(
        curve_source(circle([0., 0.], 1.), SEGMENT),
        &[target(1, segment([-2., 1.], [2., 1.]))],
    );
    assert_eq!(
        report.select_trim([0., -5.], || false),
        refused(SketchSelectionRefusal::ClosedSourceCuts { found: 1 })
    );
}
fn quarter() -> SketchIncidenceCurve {
    arc([0., 0.], 5., [1., 0.], [0., 1.], true)
}
#[test]
fn arc_trim_uses_its_own_ends_and_refuses_picks_outside_the_sweep() {
    let report = certify(
        curve_source(quarter(), SEGMENT),
        &[target(1, segment([3., -10.], [3., 10.]))],
    );
    assert_eq!(report.ordered().len(), 1);
    assert_eq!(
        report.select_trim([5., 1.], || false),
        Ok(trim(None, Some(0)))
    );
    assert_eq!(
        report.select_trim([1., 5.], || false),
        Ok(trim(Some(0), None))
    );
    for pick in [[5., -1.], [-1., -1.], [7., 0.]] {
        assert_eq!(
            report.select_trim(pick, || false),
            refused(SketchSelectionRefusal::PickOutsideSegment {
                pick: SketchTrimPick::Point(pick)
            })
        );
    }
}
#[test]
fn arc_extension_searches_the_gap_from_each_end() {
    // x = 3 and y = -3 meet the circle of radius 5 at (3, ±4) and (±4, -3).
    let targets = [
        target(1, segment([3., -10.], [3., 10.])),
        target(2, segment([-10., -3.], [10., -3.])),
    ];
    let report = certify(curve_source(quarter(), SUPPORT), &targets);
    let contact =
        |group: Option<usize>| report.contact(report.ordered()[group.unwrap()].members()[0]);
    let end = report
        .select_extension(SketchLineEnd::End, || false)
        .unwrap();
    assert_point(contact(end), [-4., -3.]);
    let start = report
        .select_extension(SketchLineEnd::Start, || false)
        .unwrap();
    assert_point(contact(start), [4., -3.]);
}
#[test]
fn full_conic_extension_and_spline_point_picks_are_refused() {
    let support = certify(curve_source(circle([0., 0.], 1.), SUPPORT), &[]);
    assert_eq!(
        support.select_extension(SketchLineEnd::End, || false),
        Err(SketchSelectionError::Refused(
            SketchSelectionRefusal::ClosedSource
        ))
    );
    let report = polyline_report();
    assert_eq!(
        report.select_trim([0., 0.], || false),
        refused(SketchSelectionRefusal::PickKind {
            pick: SketchTrimPick::Point([0., 0.])
        })
    );
    assert_eq!(
        axis_circle_report().select_trim_at(SketchTrimPick::Parameter(0.5), || false),
        refused(SketchSelectionRefusal::PickKind {
            pick: SketchTrimPick::Parameter(0.5)
        })
    );
}
fn polyline_report() -> SketchLineIncidenceReport {
    let curve = spline(
        1,
        &[[-2., 0.], [2., 0.], [2., 2.]],
        &[0., 0., 1., 2., 2.],
        [0., 2.],
    );
    certify(
        curve_source(curve, SEGMENT),
        &[target(7, circle([0., 0.], 1.))],
    )
}
#[test]
fn open_spline_trim_takes_an_exact_parameter_pick() {
    let report = polyline_report();
    let pick = |value: f64| report.select_trim_at(SketchTrimPick::Parameter(value), || false);
    assert_eq!(pick(0.5), Ok(trim(Some(0), Some(1))));
    assert_eq!(pick(1.5), Ok(trim(Some(1), None)));
    for outside in [0., 2., 3., f64::NAN] {
        assert!(matches!(
            pick(outside),
            Err(SketchSelectionError::Refused(
                SketchSelectionRefusal::PickOutsideSegment { .. }
            ))
        ));
    }
    assert_eq!(
        pick(0.25),
        refused(SketchSelectionRefusal::PickAtIncidence { group: 0 })
    );
}
#[test]
fn closed_spline_trim_wraps_across_its_seam() {
    // A unit square polyline whose two ends are the same exact point.
    let square = spline(
        1,
        &[[0., 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]],
        &[0., 0., 1., 2., 3., 4., 4.],
        [0., 4.],
    );
    let report = certify(
        curve_source(square, SEGMENT),
        &[target(1, segment([-1., 0.5], [2., 0.5]))],
    );
    assert_eq!(report.ordered().len(), 2);
    let pick = |value: f64| report.select_trim_at(SketchTrimPick::Parameter(value), || false);
    assert_eq!(pick(0.5), Ok(trim(Some(1), Some(0))));
    assert_eq!(pick(2.), Ok(trim(Some(0), Some(1))));
    assert_eq!(pick(4.), Ok(trim(Some(1), Some(0))));
}
