// SPDX-License-Identifier: GPL-2.0-only
//! Extension and trim target selection, including explicit refusals.
use super::*;

const SEGMENT: SketchLineDomain = SketchLineDomain::Segment;
const SUPPORT: SketchLineDomain = SketchLineDomain::Support;

fn extension_targets() -> Vec<SketchIncidenceTarget> {
    vec![
        target(1, circle([5., 0.], 1.)),
        target(2, segment([3., -1.], [3., 1.])),
        target(3, segment([-2., -1.], [-2., 1.])),
    ]
}
fn selected_ids(report: &SketchLineIncidenceReport, group: usize) -> Vec<u64> {
    report.ordered()[group]
        .members()
        .iter()
        .map(|member| report.contact(*member).target_id())
        .collect()
}
#[test]
fn extension_selects_the_nearest_group_beyond_each_end() {
    let report = certify(source([[0., 0.], [1., 0.]], SUPPORT), &extension_targets());
    let end = report
        .select_extension(SketchLineEnd::End, || false)
        .unwrap()
        .unwrap();
    assert_eq!(selected_ids(&report, end), [2]);
    let start = report
        .select_extension(SketchLineEnd::Start, || false)
        .unwrap()
        .unwrap();
    assert_eq!(selected_ids(&report, start), [3]);
}
#[test]
fn extension_with_nothing_beyond_completes_empty() {
    let report = certify(
        source([[0., 0.], [1., 0.]], SUPPORT),
        &[target(3, segment([-2., -1.], [-2., 1.]))],
    );
    assert_eq!(
        report.select_extension(SketchLineEnd::End, || false),
        Ok(None)
    );
}
#[test]
fn extension_returns_a_whole_tie_group() {
    let targets = [
        target(1, circle([4., 0.], 1.)),
        target(2, segment([3., -1.], [3., 1.])),
    ];
    let report = certify(source([[0., 0.], [1., 0.]], SUPPORT), &targets);
    let end = report
        .select_extension(SketchLineEnd::End, || false)
        .unwrap()
        .unwrap();
    assert_eq!(selected_ids(&report, end), [1, 2]);
}
#[test]
fn extension_refuses_an_overlap_nearer_than_the_candidate() {
    let mut targets = extension_targets();
    targets.push(target(4, segment([2., 0.], [2.5, 0.])));
    let report = certify(source([[0., 0.], [1., 0.]], SUPPORT), &targets);
    let refused = SketchSelectionError::Refused(SketchSelectionRefusal::Overlap { target_id: 4 });
    assert_eq!(
        report.select_extension(SketchLineEnd::End, || false),
        Err(refused)
    );
    assert!(
        report
            .select_extension(SketchLineEnd::Start, || false)
            .is_ok()
    );
    targets[3] = target(4, segment([4., 0.], [3.5, 0.]));
    let report = certify(source([[0., 0.], [1., 0.]], SUPPORT), &targets);
    assert!(
        report
            .select_extension(SketchLineEnd::End, || false)
            .unwrap()
            .is_some()
    );
}
#[test]
fn selections_refuse_the_wrong_domain() {
    let report = certify(source([[0., 0.], [1., 0.]], SEGMENT), &extension_targets());
    let refused = SketchSelectionError::Refused(SketchSelectionRefusal::Domain {
        required: SUPPORT,
        actual: SEGMENT,
    });
    assert_eq!(
        report.select_extension(SketchLineEnd::End, || false),
        Err(refused)
    );
}
#[test]
fn trim_selects_the_groups_around_an_exactly_projected_pick() {
    let targets = [
        target(1, segment([1., -1.], [1., 1.])),
        target(2, segment([3., -1.], [3., 1.])),
    ];
    let report = certify(source([[0., 0.], [4., 0.]], SEGMENT), &targets);
    let around = report.select_trim([2., 0.3], || false).unwrap();
    assert_eq!(
        around,
        SketchTrimSelection {
            before: Some(0),
            after: Some(1)
        }
    );
    let left = report.select_trim([0.5, -7.], || false).unwrap();
    assert_eq!(
        left,
        SketchTrimSelection {
            before: None,
            after: Some(0)
        }
    );
}
#[test]
fn trim_refuses_picks_at_incidences_outside_the_segment_or_on_overlaps() {
    let targets = [
        target(1, segment([1., -1.], [1., 1.])),
        target(2, segment([2.5, 0.], [3.5, 0.])),
    ];
    let report = certify(source([[0., 0.], [4., 0.]], SEGMENT), &targets);
    let at = SketchSelectionError::Refused(SketchSelectionRefusal::PickAtIncidence { group: 0 });
    assert_eq!(report.select_trim([1., 5.], || false), Err(at));
    let outside = SketchSelectionError::Refused(SketchSelectionRefusal::PickOutsideSegment {
        pick: SketchTrimPick::Point([5., 0.]),
    });
    assert_eq!(report.select_trim([5., 0.], || false), Err(outside));
    let overlap = SketchSelectionError::Refused(SketchSelectionRefusal::Overlap { target_id: 2 });
    assert_eq!(report.select_trim([2., 0.], || false), Err(overlap));
    assert!(report.select_trim([0.5, 0.], || false).is_ok());
}
#[test]
fn cancelled_selection_is_incomplete_not_empty() {
    // The root x = sqrt(3)/2 is irrational; a pick at its binary64 rounding lies
    // inside the certified bracket, so the comparison must refine it.
    let report = certify(
        source([[-2., 0.5], [2., 0.5]], SEGMENT),
        &[target(1, circle([0., 0.], 1.))],
    );
    let result = report.select_trim([3f64.sqrt() / 2., 0.5], || true);
    assert!(
        matches!(result, Err(SketchSelectionError::Incomplete(_))),
        "{result:?}"
    );
}
#[test]
fn unbounded_collinear_overlap_refuses_both_edits() {
    let collinear = target(7, infinite([9., 0.], [8., 0.]));
    let refused = SketchSelectionError::Refused(SketchSelectionRefusal::Overlap { target_id: 7 });
    let support = certify(
        source([[0., 0.], [1., 0.]], SUPPORT),
        std::slice::from_ref(&collinear),
    );
    assert_eq!(
        support.select_extension(SketchLineEnd::Start, || false),
        Err(refused.clone())
    );
    let segment = certify(source([[0., 0.], [1., 0.]], SEGMENT), &[collinear]);
    assert_eq!(segment.select_trim([0.5, 1.], || false), Err(refused));
}
#[test]
fn overlaps_behind_or_outside_the_edited_range_do_not_refuse() {
    let targets = [
        target(1, segment([3., -1.], [3., 1.])),
        target(2, segment([-1., 0.], [-3., 0.])),
        target(3, segment([0.5, -1.], [0.5, 1.])),
    ];
    let report = certify(source([[0., 0.], [4., 0.]], SUPPORT), &targets);
    let start = report.select_extension(SketchLineEnd::Start, || false);
    let refused = SketchSelectionError::Refused(SketchSelectionRefusal::Overlap { target_id: 2 });
    assert_eq!(
        start,
        Err(refused),
        "the overlap is the only thing before the start"
    );
    assert!(
        report
            .select_extension(SketchLineEnd::End, || false)
            .unwrap()
            .is_none()
    );
    let report = certify(source([[0., 0.], [4., 0.]], SEGMENT), &targets);
    let around = report.select_trim([2., 0.], || false).unwrap();
    assert_eq!(
        around,
        SketchTrimSelection {
            before: Some(0),
            after: Some(1)
        }
    );
    let nonfinite = report.select_trim([f64::NAN, 0.], || false);
    assert!(matches!(
        nonfinite,
        Err(SketchSelectionError::Refused(
            SketchSelectionRefusal::PickOutsideSegment { .. }
        ))
    ));
}
