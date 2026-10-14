// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkernel::standard::Failure;

struct NamedPublicationProgress {
    fail: bool,
    stop: Option<usize>,
    seen: Vec<PrismPublicationCheckpoint>,
}
impl PrismPublicationProgress for NamedPublicationProgress {
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.seen.push(point);
        if self.fail {
            return Err(Failure::new(
                "authored callback 0 failed: expected uninterrupted publication",
            ));
        }
        Ok(self.stop == Some(point.ordinal))
    }
}
#[test]
fn one_ordinal_sequence_keeps_cancellation_terminal_across_stages() {
    let mut progress = NamedPublicationProgress {
        fail: false,
        stop: Some(1),
        seen: Vec::new(),
    };
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    assert!(!callbacks.cancelled(PrismPublicationPhase::Admission));
    assert!(callbacks.cancelled(PrismPublicationPhase::Join));
    assert!(callbacks.cancelled(PrismPublicationPhase::Commit));
    let stop = callbacks.take_stop().unwrap();
    assert_eq!(stop.checkpoint.phase, PrismPublicationPhase::Join);
    assert_eq!(stop.checkpoint.ordinal, 1);
    assert!(matches!(stop.cause, PublicationCallbackCause::Cancelled));
    assert!(callbacks.cancelled(PrismPublicationPhase::Measures));
    assert_eq!(progress.seen.len(), 2);
}
#[test]
fn callback_failure_retains_typed_cause_and_checkpoint() {
    let mut progress = NamedPublicationProgress {
        fail: true,
        stop: None,
        seen: Vec::new(),
    };
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    assert!(callbacks.cancelled(PrismPublicationPhase::History));
    let stop = callbacks.take_stop().unwrap();
    assert_eq!(stop.checkpoint.phase, PrismPublicationPhase::History);
    assert!(matches!(stop.cause, PublicationCallbackCause::Failed(_)));
    fn send<T: Send>() {}
    send::<PublicationCallbackStop>();
}

#[test]
fn exhausted_checkpoint_counter_refuses_before_callback() {
    let mut progress = NamedPublicationProgress {
        fail: false,
        stop: None,
        seen: Vec::new(),
    };
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    callbacks.ordinal = usize::MAX;
    assert!(callbacks.cancelled(PrismPublicationPhase::Commit));
    assert!(matches!(
        callbacks.take_stop().unwrap().cause,
        PublicationCallbackCause::Exhausted
    ));
    assert!(progress.seen.is_empty());
}
