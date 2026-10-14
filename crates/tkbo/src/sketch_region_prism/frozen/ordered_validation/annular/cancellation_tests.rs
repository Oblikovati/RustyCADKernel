// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region_prism::frozen::annular_join::operation_tests::joined;
#[test]
fn every_ordered_checkpoint_preserves_cancellation_or_native_cause_then_reuse() {
    let join = joined();
    let mut progress = RecordedAnnularProgress::default();
    let expected = validate_annular_prism(&join, options(), &mut progress)
        .ok()
        .unwrap()
        .replay_bytes();
    for stop in 1..=progress.points.len() {
        for fail in [false, true] {
            assert_cutoff(&join, &progress.points, stop, fail);
        }
    }
    assert_eq!(complete(&join).replay_bytes(), expected);
}
fn assert_cutoff(
    join: &AnnularFrozenDomain,
    expected: &[AnnularValidationCheckpoint],
    stop: usize,
    fail: bool,
) {
    let mut progress = RecordedAnnularProgress {
        stop: Some(stop),
        fail,
        ..Default::default()
    };
    let failure = validate_annular_prism(join, options(), &mut progress)
        .err()
        .unwrap();
    assert_eq!(progress.points, expected[..stop]);
    assert_eq!(failure.checkpoint, expected[stop - 1]);
    assert_eq!(failure.source.as_ref(), join.replay_bytes());
    let class = if fail {
        OrderedPrismFailureClass::Failed
    } else {
        OrderedPrismFailureClass::Cancelled
    };
    assert_eq!(failure.class, class);
    assert_callback(&failure.cause, fail);
    assert_stopped_trace(&failure);
}
fn assert_callback(cause: &AnnularValidationCause, failed: bool) {
    let AnnularValidationCause::Callback { stop, downstream } = cause else {
        panic!("outer callback retained");
    };
    assert!(downstream.is_some());
    match (&stop.cause, failed) {
        (AnnularValidationCallbackCause::Cancelled, false) => (),
        (AnnularValidationCallbackCause::Failed(cause), true) => {
            assert_eq!(cause.message(), "injected annular validator native failure")
        }
        _ => panic!("exact original callback cause expected"),
    }
}
fn assert_stopped_trace(failure: &AnnularValidationFailure) {
    for stage in OrderedPrismStage::ALL {
        let expected = match stage.index().cmp(&failure.checkpoint.stage.index()) {
            std::cmp::Ordering::Less => OrderedPrismStageStatus::Passed,
            std::cmp::Ordering::Equal => failure.class.into(),
            std::cmp::Ordering::Greater => OrderedPrismStageStatus::Blocked,
        };
        assert_eq!(failure.trace.status(stage), expected);
    }
}
#[test]
fn context_ordinal_exhaustion_is_incomplete_without_callback_or_saturation() {
    let mut progress = RecordedAnnularProgress::default();
    let mut context = AnnularValidationContext::new(&mut progress);
    context.current.ordinal = usize::MAX;
    assert!(context.poll(AnnularValidationCheck::Commit));
    let stop = context.accept(Ok(())).err().unwrap();
    assert_eq!(stop.class, OrderedPrismFailureClass::Incomplete);
    let AnnularValidationCause::Callback { stop, downstream } = stop.cause else {
        panic!("exhaustion");
    };
    assert!(matches!(
        stop.cause,
        AnnularValidationCallbackCause::Exhausted
    ));
    assert_eq!(stop.checkpoint.ordinal, usize::MAX);
    assert!(downstream.is_none());
    drop(context);
    assert!(progress.points.is_empty());
}
#[test]
fn first_callback_stop_is_not_overwritten_by_later_child_polling() {
    let mut progress = RecordedAnnularProgress {
        stop: Some(1),
        fail: true,
        ..Default::default()
    };
    let mut context = AnnularValidationContext::new(&mut progress);
    assert!(context.poll(AnnularValidationCheck::Admission));
    let first = context.current;
    assert!(context.poll(AnnularValidationCheck::Commit));
    assert_eq!(context.current, first);
    let stop: ValidationCheck<()> = context.accept(Err(ValidationStop::cancelled()));
    assert_callback(&stop.err().unwrap().cause, true);
    drop(context);
    assert_eq!(progress.points.len(), 1);
}
