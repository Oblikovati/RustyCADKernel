// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn run_input(captured: &mut CapturedBezierWireStore, input: CapturedBezierWireInput) {
    captured
        .classify(&[input_occurrence(1, input)], || false)
        .unwrap();
}
#[test]
fn exact_scalar_payload_and_physical_source_are_both_required_for_admission() {
    let mut captured = store(100);
    run_input(&mut captured, occurrence(7, 1, false).input);
    run_input(&mut captured, occurrence(8, 1, false).input);
    let mut changed = CUBIC;
    changed[0][0] = -0.;
    run_input(
        &mut captured,
        CapturedBezierWireInput::capture(7, &changed, None, [0., 1.]),
    );
    run_input(
        &mut captured,
        CapturedBezierWireInput::capture(7, &CUBIC, Some(&[1.; 4]), [0., 1.]),
    );
    run_input(
        &mut captured,
        CapturedBezierWireInput::capture(7, &CUBIC, None, [0.5, 0.5]),
    );
    assert_eq!(captured.proofs().len(), 5);
}
#[test]
fn failed_payload_does_not_replace_an_existing_complete_proof() {
    let mut captured = store(100);
    run_input(&mut captured, occurrence(7, 1, false).input);
    let mut invalid = CUBIC;
    invalid[1][0] = f64::INFINITY;
    let input = CapturedBezierWireInput::capture(7, &invalid, None, [0., 1.]);
    let failed = captured
        .classify(&[input_occurrence(2, input)], || false)
        .unwrap_err();
    assert_coordinate_failure(&failed);
    assert_eq!(captured.proofs().len(), 1);
    assert_eq!(
        evaluate_cubic(&mut captured, 7, 3, true)
            .work()
            .reused_certificates,
        1
    );
}
#[test]
fn duplicate_ids_and_warm_cancellation_preserve_prior_complete_proof() {
    let mut captured = store(100);
    run_input(&mut captured, occurrence(7, 1, false).input);
    let duplicate = captured
        .classify(&[occurrence(7, 3, false), occurrence(7, 3, true)], || false)
        .unwrap_err();
    assert_eq!(duplicate.cause(), &BezierWireError::DuplicateId { id: 3 });
    let cancelled = captured
        .classify(&[occurrence(7, 4, false)], || true)
        .unwrap_err();
    assert_eq!(cancelled.cause(), &BezierWireError::Cancelled { id: 4 });
    assert_eq!(cancelled.work().new_bisections, 0);
    assert_eq!(captured.proofs().len(), 1);
    assert_eq!(
        evaluate_cubic(&mut captured, 7, 5, false)
            .work()
            .reused_certificates,
        1
    );
}
#[test]
fn changed_ray_or_policy_starts_a_distinct_owned_operation() {
    let mut captured = store(100);
    run_input(&mut captured, occurrence(7, 1, false).input);
    let mut alternate =
        CapturedBezierWireStore::new([[2., 2.], [3., 2.]], RootIsolationOptions::new(0.001));
    let other = evaluate_cubic(&mut alternate, 7, 1, false);
    assert_eq!(other.location(), &BezierWireLocation::Outside);
    assert_eq!(other.work().reused_certificates, 0);
    let failed = store(0)
        .classify(&[occurrence(7, 1, false)], || false)
        .unwrap_err();
    assert_eq!(failed.work().reused_certificates, 0);
    assert_eq!(captured.proofs().len(), 1);
}

fn assert_coordinate_failure(failed: &CapturedBezierWireFailure) {
    assert!(matches!(
        failed.cause(),
        BezierWireError::Segment {
            id: 2,
            cause: BezierRayError::Line(BezierLineError::Coordinate { .. })
        }
    ));
    assert_eq!(failed.work().new_bisections, 0);
}
