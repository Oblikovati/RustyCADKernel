// SPDX-License-Identifier: GPL-2.0-only
use super::*;

fn refused(requested: AnnularMeasureOptions) -> Box<AnnularMeasureFailure> {
    let join = joined();
    let error = inspect_annular_measures(&join, requested, || false)
        .err()
        .unwrap();
    assert_eq!(error.requested, requested);
    assert_eq!(error.binding.as_ref(), join.replay_bytes());
    assert_eq!(
        error.classification(),
        JoinedMeasureFailureClass::Incomplete
    );
    error
}
#[test]
fn behavior_version_is_refused_before_callback_or_partial_evidence() {
    let join = joined();
    let mut requested = options();
    requested.version = 1;
    let failure = inspect_annular_measures(&join, requested, || panic!("unadmitted callback"))
        .err()
        .unwrap();
    assert!(matches!(
        failure.cause,
        AnnularMeasureCause::Version {
            actual: 1,
            expected: 2
        }
    ));
    assert!(failure.partial.faces.is_empty());
    assert_eq!(failure.phase, AnnularMeasurePhase::Admission);
}
#[test]
fn original_side_policy_limits_are_preserved_before_clamping() {
    for side in invalid_side_policies() {
        let mut requested = options();
        requested.side_area = side;
        requested.maximum_side_subdivisions = 0;
        let failure = refused(requested);
        assert!(
            matches!(failure.cause, AnnularMeasureCause::OriginalSidePolicy(PolynomialSideFailure::Policy(actual)) if actual == side)
        );
        assert!(failure.partial.faces.is_empty());
    }
}
fn invalid_side_policies() -> Vec<PolynomialSideOptions> {
    let original = options().side_area;
    vec![
        PolynomialSideOptions {
            behavior_version: 2,
            ..original
        },
        PolynomialSideOptions {
            max_degree: 17,
            ..original
        },
        PolynomialSideOptions {
            max_subdivisions: 65537,
            ..original
        },
        PolynomialSideOptions {
            absolute_error_limit: -1.,
            ..original
        },
    ]
}
#[test]
fn per_side_zero_cap_is_not_widened_by_remaining_aggregate_budget() {
    let mut requested = options();
    requested.side_area.max_subdivisions = 0;
    let failure = refused(requested);
    let AnnularMeasureCause::Side(side) = failure.cause else {
        panic!("typed side work failure")
    };
    assert_eq!(side.requested_options, requested.side_area);
    assert_eq!(side.effective_options, Some(requested.side_area));
    assert_eq!(side.work.subdivisions, 0);
    assert!(matches!(
        *side.cause,
        PolynomialSideFailure::WorkLimit { requested: 0, .. }
    ));
}
#[test]
fn cap_and_sweep_error_caps_remain_distinct_and_keep_completed_prefix() {
    let mut cap_request = options();
    cap_request.cap_area.absolute_error_limit = 0.;
    let cap = refused(cap_request);
    assert_eq!(cap.phase, AnnularMeasurePhase::Caps);
    assert!(matches!(cap.cause, AnnularMeasureCause::Cap(_)));
    assert!(cap.partial.faces.is_empty());
    let mut sweep_request = options();
    sweep_request.signed_sweep.absolute_error_limit = 0.;
    let sweep = refused(sweep_request);
    assert_eq!(sweep.phase, AnnularMeasurePhase::Sweep);
    assert!(matches!(sweep.cause, AnnularMeasureCause::Sweep(_)));
    assert_eq!(sweep.partial.faces.len(), 2);
    assert!(sweep.partial.signed_sweep.is_none());
}
