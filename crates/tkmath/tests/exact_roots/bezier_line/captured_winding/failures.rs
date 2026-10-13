// SPDX-License-Identifier: GPL-2.0-only
use super::*;
fn custom(id: u64, controls: &[[f64; 2]], weights: Option<&[f64]>) -> CapturedBezierWireOccurrence {
    CapturedBezierWireOccurrence {
        id,
        reversed: false,
        input: CapturedBezierWireInput::capture(id, controls, weights, [0., 1.]),
    }
}
#[test]
fn failed_budget_is_charged_without_admitting_partial_proof() {
    let mut captured = store(1);
    let failed = captured
        .classify(&[occurrence(7, 1, false)], || false)
        .unwrap_err();
    assert_eq!(failed.work().new_bisections, 1);
    assert_eq!(failed.work().operation_bisections, 1);
    assert!(captured.proofs().is_empty());
    let next = captured
        .classify(&[occurrence(7, 2, false)], || false)
        .unwrap_err();
    assert_eq!(next.work().new_bisections, 0);
    assert_eq!(next.work().operation_bisections, 1);
}
#[test]
fn denominator_failure_precedes_an_open_join() {
    let first = custom(1, &[[0., 0.], [1., 0.]], None);
    let pole = custom(2, &[[8., 0.], [9., 0.]], Some(&[1., -1.]));
    let mut captured =
        CapturedBezierWireStore::new([[0., 0.], [1., 0.]], RootIsolationOptions::new(0.001));
    let failed = captured.classify(&[first, pole], || false).unwrap_err();
    assert!(matches!(
        failed.cause(),
        BezierWireError::Segment {
            id: 2,
            cause: BezierRayError::Line(BezierLineError::DenominatorRoots(_))
        }
    ));
    assert_eq!(failed.work().operation_bisections, 1);
    assert_eq!(captured.proofs().len(), 1);
}
#[test]
fn open_join_precedes_a_later_numerator_budget_failure() {
    let first = custom(1, &[[0., 0.]], None);
    let later = custom(2, &[[3., 0.], [4., 1.], [4., -2.], [3., 0.]], None);
    let options = RootIsolationOptions {
        max_bisections: 0,
        ..RootIsolationOptions::new(0.001)
    };
    let failed = CapturedBezierWireStore::new([[0., 0.], [1., 0.]], options)
        .classify(&[first, later], || false)
        .unwrap_err();
    assert_eq!(
        failed.cause(),
        &BezierWireError::OpenJoin { from: 1, to: 2 }
    );
    assert_eq!(failed.work().operation_bisections, 0);
}
#[test]
fn cancellation_after_completed_splits_charges_work_and_preserves_old_proofs() {
    let (mut captured, failed) = cancelled_after_split();
    assert!(failed.to_string().contains("cancel"));
    assert!(captured.proofs().is_empty());
    let recovered = evaluate_cubic(&mut captured, 7, 2, false);
    assert!(recovered.work().operation_bisections > recovered.work().new_bisections);
}

fn cancelled_after_split() -> (CapturedBezierWireStore, CapturedBezierWireFailure) {
    let mut found = None;
    for stop in 1..=700 {
        let mut captured = store(100);
        let mut polls = 0;
        let result = captured.classify(&[occurrence(7, 1, false)], || {
            polls += 1;
            polls == stop
        });
        if let Err(failed) = result
            && failed.work().new_bisections > 0
        {
            found = Some((captured, failed));
            break;
        }
    }
    found.expect("expected cancellation after actual subdivision")
}
