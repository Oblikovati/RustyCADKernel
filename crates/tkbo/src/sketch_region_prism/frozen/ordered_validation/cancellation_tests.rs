// SPDX-License-Identifier: GPL-2.0-only
use super::{test_support::*, *};

#[test]
fn every_observed_checkpoint_cancellation_preserves_order_and_input() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let source = join.replay_bytes().to_vec();
    let points = observed(&join);
    for point in points {
        let mut progress = NamedValidationProgress {
            stop: Some(point.ordinal),
            ..Default::default()
        };
        let refused = failure(validate_joined_prism(&join, options(), &mut progress));
        check_stopped(&refused, point, OrderedPrismFailureClass::Cancelled);
        assert_eq!(progress.points.last(), Some(&point));
        assert_eq!(join.replay_bytes(), source);
    }
    assert!(complete(&join).binds(&join));
}
#[test]
fn every_callback_failure_remains_incomplete_with_original_typed_cause() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    for point in observed(&join) {
        let mut progress = NamedValidationProgress {
            stop: Some(point.ordinal),
            fail: true,
            ..Default::default()
        };
        let refused = failure(validate_joined_prism(&join, options(), &mut progress));
        check_stopped(&refused, point, OrderedPrismFailureClass::Incomplete);
        assert!(matches!(refused.cause, OrderedPrismCause::Callback(_)));
    }
}
fn observed(join: &JoinedPrismProof) -> Vec<OrderedPrismCheckpoint> {
    let mut progress = NamedValidationProgress::default();
    assert!(matches!(
        validate_joined_prism(join, options(), &mut progress),
        OrderedPrismValidationOutcome::CompleteValidation(_)
    ));
    assert_eq!(
        progress.points.last().unwrap().check,
        OrderedPrismCheck::Commit
    );
    assert!(
        progress
            .points
            .windows(2)
            .all(|pair| pair[0].ordinal + 1 == pair[1].ordinal)
    );
    progress.points
}
fn check_stopped(
    refused: &OrderedPrismValidationFailure,
    point: OrderedPrismCheckpoint,
    class: OrderedPrismFailureClass,
) {
    assert_eq!(refused.checkpoint, point);
    assert_eq!(refused.class, class);
    for stage in OrderedPrismStage::ALL {
        let expected = match stage.index().cmp(&point.stage.index()) {
            std::cmp::Ordering::Less => OrderedPrismStageStatus::Passed,
            std::cmp::Ordering::Equal => class.into(),
            std::cmp::Ordering::Greater => OrderedPrismStageStatus::Blocked,
        };
        assert_eq!(refused.trace.status(stage), expected);
    }
}
