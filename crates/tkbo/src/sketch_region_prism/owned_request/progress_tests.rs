// SPDX-License-Identifier: GPL-2.0-only
use super::{diagnostics::*, pipeline::*, progress::*, test_fixture::*};
#[test]
fn every_observed_callback_cancellation_retains_owned_inputs_and_allows_reuse() {
    let mut baseline = RequestObserver::default();
    assert!(prepare_owned_polynomial_prism(request(), &mut baseline).is_ok());
    for stop in 0..baseline.seen.len() {
        assert_cancelled(&baseline.seen, stop);
    }
    assert!(prepare_owned_polynomial_prism(request(), &mut RequestObserver::default()).is_ok());
    println!("OWNED_REQUEST_CALLBACK_COUNT={}", baseline.seen.len());
}
#[test]
fn every_child_stage_callback_error_keeps_failure_and_terminal_checkpoint() {
    let mut baseline = RequestObserver::default();
    assert!(prepare_owned_polynomial_prism(request(), &mut baseline).is_ok());
    for checkpoint in first_stage_checkpoints(&baseline.seen) {
        let mut observer = RequestObserver {
            fail: Some(checkpoint.ordinal),
            ..RequestObserver::default()
        };
        let failure = prepare_owned_polynomial_prism(request(), &mut observer)
            .err()
            .unwrap();
        assert_eq!(failure.class, OwnedPreparationClass::Failed);
        let stop = failure.callback.unwrap();
        assert_eq!(stop.checkpoint, checkpoint);
        assert!(matches!(stop.cause, OwnedPrismCallbackCause::Failed(_)));
        assert_eq!(observer.seen.last(), Some(&checkpoint));
    }
}
fn first_stage_checkpoints(seen: &[OwnedPrismCheckpoint]) -> Vec<OwnedPrismCheckpoint> {
    let mut unique: Vec<OwnedPrismCheckpoint> = vec![];
    for checkpoint in seen {
        if !unique.iter().any(|p| p.phase == checkpoint.phase) {
            unique.push(*checkpoint);
        }
    }
    unique
}
#[test]
fn missing_private_state_is_typed_and_cannot_fabricate_a_prepared_stack() {
    assert!(matches!(
        super::diagnostics::local::<u8>(None, "original source"),
        Err(OwnedPreparationCause::MissingLocalState {
            expected: "original source"
        })
    ));
    assert_eq!(
        super::diagnostics::local(Some(9), "original source").ok(),
        Some(9)
    );
}

fn assert_cancelled(baseline: &[OwnedPrismCheckpoint], ordinal: usize) {
    let mut observer = RequestObserver {
        stop: Some(ordinal),
        ..RequestObserver::default()
    };
    let failure = prepare_owned_polynomial_prism(request(), &mut observer)
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Cancelled);
    assert_eq!(observer.seen, baseline[..=ordinal]);
    assert!(matches!(
        failure.callback.unwrap().cause,
        OwnedPrismCallbackCause::Cancelled
    ));
    assert_eq!(
        failure.retained.request.sources[0].world_poles,
        request().sources[0].world_poles
    );
}
