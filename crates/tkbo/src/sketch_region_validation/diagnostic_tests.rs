// SPDX-License-Identifier: GPL-2.0-only
//! Typed defects, unknown coverage and complete parent sets must survive adapters.
use super::outcome::{StageCoverage, ValidationCause, ValidationStage, attributed_occurrence};
use super::statuses::status_cause;
use rustycad_tktopalgo::brep_check::{BRepCheckAnalyzer, BRepCheckStatus, ListOfStatus};

fn statuses(values: &[BRepCheckStatus]) -> ListOfStatus {
    let mut result = ListOfStatus::default();
    for status in values {
        result.append(*status);
    }
    result
}

#[test]
fn empty_statuses_and_check_failure_are_incomplete() {
    let empty = status_cause(ListOfStatus::default()).unwrap();
    assert_eq!(empty.0, StageCoverage::Incomplete);
    assert!(matches!(empty.1, ValidationCause::EmptyStatuses));
    let failed = status_cause(statuses(&[BRepCheckStatus::CheckFail])).unwrap();
    assert_eq!(failed.0, StageCoverage::Incomplete);
}

#[test]
fn geometric_defects_keep_native_status_order() {
    let values = [
        BRepCheckStatus::InvalidCurveOnSurface,
        BRepCheckStatus::InvalidSameParameterFlag,
    ];
    let (coverage, cause) = status_cause(statuses(&values)).unwrap();
    assert_eq!(coverage, StageCoverage::Failed);
    let ValidationCause::NativeStatuses(actual) = cause else {
        panic!("expected retained native statuses")
    };
    assert_eq!(actual, values);
    assert!(status_cause(statuses(&[BRepCheckStatus::NoError])).is_none());
}

#[test]
fn occurrence_parent_sets_are_complete_sorted_and_orientation_specific() {
    use rustycad_tkg3d::top_abs::Orientation;
    let face = super::tests::box_face();
    let sources = [
        super::source_relation_test_support::test_source_relation(&face, &[7, 3]),
        super::source_relation_test_support::test_source_relation(&face, &[7, 11]),
    ];
    let related = attributed_occurrence(&face, &sources);
    assert_eq!(related.source_parents, Some(vec![3, 7, 11]));
    assert_eq!(
        attributed_occurrence(&face.oriented(Orientation::Internal), &sources).source_parents,
        None
    );
}

#[test]
fn absent_analyzer_result_preserves_exact_affected_occurrence() {
    let first = super::tests::box_face();
    let second = super::tests::box_face();
    let analyzer = BRepCheckAnalyzer::new(&first, false, false, true).unwrap();
    let findings = super::inspection::inspect_candidate(
        &analyzer,
        &second,
        super::outcome::BodyKind::PlanarRegion,
        &[],
        ValidationStage::Topology,
    );
    assert_eq!(findings[0].coverage, StageCoverage::Incomplete);
    assert!(findings[0].affected.shape.is_equal(&second));
    assert!(matches!(
        findings[0].cause,
        ValidationCause::NativeFailure(_)
    ));
}

#[test]
fn empty_body_status_is_a_failed_topological_body() {
    use rustycad_tkbrep::{brep::BRepBuilder, topods::Face};
    let mut face = Face::default();
    BRepBuilder::default().make_face(&mut face);
    let report =
        super::validate_owned_candidate(&face, super::outcome::BodyKind::PlanarRegion, &[]);
    assert_eq!(report.topology, StageCoverage::Failed);
    assert_eq!(report.geometry, StageCoverage::Blocked);
    assert!(!report.complete());
}
