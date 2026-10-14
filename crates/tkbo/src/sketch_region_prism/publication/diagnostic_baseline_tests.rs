// SPDX-License-Identifier: GPL-2.0-only
//! ADR0048 pins retained publication facts before adding invocation-bound projections.
use super::{progress::PublicationCallbacks, stage_error::finish_child, *};
use rustycad_tkernel::standard::Failure;

struct RefusingPublicationCheckpoint {
    failure: Option<Failure>,
}
impl PrismPublicationProgress for RefusingPublicationCheckpoint {
    fn checkpoint(&mut self, _: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        match self.failure.take() {
            Some(failure) => Err(failure),
            None => Ok(true),
        }
    }
}
fn child_failure(class: PrismPublicationClass) -> PublicationStageFailure {
    PublicationStageFailure {
        phase: PrismPublicationPhase::Measures,
        class,
        cause: PublicationStageCause::Binding {
            proof: "original child binding",
        },
    }
}
fn interrupted_child(
    class: PrismPublicationClass,
    observer: &mut RefusingPublicationCheckpoint,
) -> PublicationStageFailure {
    let mut callbacks = PublicationCallbacks::new(observer);
    assert!(callbacks.cancelled(PrismPublicationPhase::Join));
    finish_child::<()>(Err(child_failure(class)), &mut callbacks)
        .err()
        .unwrap()
}
fn assert_child_binding(cause: Option<Box<PublicationStageCause>>) {
    let Some(cause) = cause else {
        panic!("received absent downstream cause; expected original binding cause");
    };
    assert!(matches!(
        *cause,
        PublicationStageCause::Binding {
            proof: "original child binding"
        }
    ));
}
#[test]
fn callback_replaces_child_header_but_retains_original_binding_payload() {
    for class in [
        PrismPublicationClass::Failed,
        PrismPublicationClass::Incomplete,
        PrismPublicationClass::Cancelled,
    ] {
        let mut observer = RefusingPublicationCheckpoint { failure: None };
        let failure = interrupted_child(class, &mut observer);
        assert_eq!(failure.phase, PrismPublicationPhase::Join);
        assert_eq!(failure.class, PrismPublicationClass::Cancelled);
        assert_cancelled_child(failure.cause);
    }
}
fn assert_cancelled_child(cause: PublicationStageCause) {
    let PublicationStageCause::Callback { stop, downstream } = cause else {
        panic!("received non-callback cause; expected retained callback and child");
    };
    assert_eq!(stop.checkpoint.phase, PrismPublicationPhase::Join);
    assert_eq!(stop.checkpoint.ordinal, 0);
    assert!(matches!(stop.cause, PublicationCallbackCause::Cancelled));
    assert_child_binding(downstream);
}
#[test]
fn native_observer_failure_keeps_exact_native_provenance_and_downstream_payload() {
    let original = Failure::out_of_range("original observer message");
    let mut observer = RefusingPublicationCheckpoint {
        failure: Some(original.clone()),
    };
    let failure = interrupted_child(PrismPublicationClass::Incomplete, &mut observer);
    assert_eq!(failure.class, PrismPublicationClass::Failed);
    let PublicationStageCause::Callback { stop, downstream } = failure.cause else {
        panic!("received non-callback cause; expected original native failure");
    };
    let PublicationCallbackCause::Failed(actual) = stop.cause else {
        panic!("received non-native cause; expected original native failure");
    };
    assert_eq!(actual, original);
    assert_child_binding(downstream);
}
#[test]
fn no_callback_stop_preserves_actual_child_header() {
    let mut observer = RefusingPublicationCheckpoint { failure: None };
    let mut callbacks = PublicationCallbacks::new(&mut observer);
    let original = child_failure(PrismPublicationClass::Failed);
    let failure = finish_child::<()>(Err(original), &mut callbacks)
        .err()
        .unwrap();
    assert_eq!(failure.phase, PrismPublicationPhase::Measures);
    assert_eq!(failure.class, PrismPublicationClass::Failed);
    assert_child_binding(Some(Box::new(failure.cause)));
}
