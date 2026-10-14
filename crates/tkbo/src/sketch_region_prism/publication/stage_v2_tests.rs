// SPDX-License-Identifier: GPL-2.0-only
//! Independently expected V2 child provenance, including class differing from callback precedence.
use super::*;
use rustycad_tkernel::standard::Failure;

#[derive(Clone, Copy)]
enum ExpectedStop {
    Cancelled,
    Native,
    Exhausted,
}
fn callback_stop(expected: ExpectedStop) -> PublicationCallbackStop {
    let cause = match expected {
        ExpectedStop::Cancelled => PublicationCallbackCause::Cancelled,
        ExpectedStop::Native => {
            PublicationCallbackCause::Failed(Failure::out_of_range("native V2 stop"))
        }
        ExpectedStop::Exhausted => PublicationCallbackCause::Exhausted,
    };
    PublicationCallbackStop {
        checkpoint: PrismPublicationCheckpoint {
            phase: PrismPublicationPhase::Join,
            ordinal: 7,
        },
        cause,
    }
}
fn original_child(class: PrismPublicationClass) -> PublicationStageFailure {
    PublicationStageFailure {
        phase: PrismPublicationPhase::Measures,
        class,
        cause: PublicationStageCause::Binding {
            proof: "independent original child",
        },
    }
}
fn assert_original_child(child: PublicationStageFailure, class: PrismPublicationClass) {
    assert_eq!(child.phase, PrismPublicationPhase::Measures);
    assert_eq!(child.class, class);
    assert!(matches!(
        child.cause,
        PublicationStageCause::Binding {
            proof: "independent original child"
        }
    ));
}
fn assert_stop(stop: PublicationCallbackStop, expected: ExpectedStop) {
    assert_eq!(stop.checkpoint.phase, PrismPublicationPhase::Join);
    assert_eq!(stop.checkpoint.ordinal, 7);
    match (stop.cause, expected) {
        (PublicationCallbackCause::Cancelled, ExpectedStop::Cancelled)
        | (PublicationCallbackCause::Exhausted, ExpectedStop::Exhausted) => (),
        (PublicationCallbackCause::Failed(value), ExpectedStop::Native) => {
            assert_eq!(value, Failure::out_of_range("native V2 stop"))
        }
        _ => panic!("received different callback cause; expected selected original stop"),
    }
}
#[test]
fn v2_retains_every_actual_child_class_phase_and_payload_for_each_stop_kind() {
    for class in [
        PrismPublicationClass::Failed,
        PrismPublicationClass::Incomplete,
        PrismPublicationClass::Cancelled,
    ] {
        for expected in [
            ExpectedStop::Cancelled,
            ExpectedStop::Native,
            ExpectedStop::Exhausted,
        ] {
            assert_retained_class(class, expected);
        }
    }
}
fn assert_retained_class(class: PrismPublicationClass, expected: ExpectedStop) {
    let cause = callback_cause(Some(original_child(class)), callback_stop(expected), true);
    let PublicationStageCause::CallbackWithChild { stop, downstream } = cause else {
        panic!("received legacy callback cause; expected V2 complete child");
    };
    assert_stop(*stop, expected);
    assert_original_child(*downstream.unwrap(), class);
}
#[test]
fn v2_does_not_invent_a_child_when_callback_refuses_successful_child() {
    let cause = callback_cause(None, callback_stop(ExpectedStop::Native), true);
    let PublicationStageCause::CallbackWithChild { stop, downstream } = cause else {
        panic!("received legacy callback cause; expected V2 absent child");
    };
    assert_stop(*stop, ExpectedStop::Native);
    assert!(downstream.is_none());
}
#[test]
fn nested_v2_children_preserve_each_header_without_flattening() {
    let cause = callback_cause(
        Some(original_child(PrismPublicationClass::Failed)),
        callback_stop(ExpectedStop::Cancelled),
        true,
    );
    let nested = PublicationStageFailure {
        phase: PrismPublicationPhase::Validation,
        class: PrismPublicationClass::Cancelled,
        cause,
    };
    let outer = callback_cause(Some(nested), callback_stop(ExpectedStop::Native), true);
    let PublicationStageCause::CallbackWithChild { stop, downstream } = outer else {
        panic!("expected V2 outer callback")
    };
    assert_stop(*stop, ExpectedStop::Native);
    assert_nested_header(*downstream.unwrap());
}
fn assert_nested_header(nested: PublicationStageFailure) {
    assert_eq!(nested.phase, PrismPublicationPhase::Validation);
    assert_eq!(nested.class, PrismPublicationClass::Cancelled);
    let PublicationStageCause::CallbackWithChild { stop, downstream } = nested.cause else {
        panic!("received flattened child; expected nested V2 callback");
    };
    assert_stop(*stop, ExpectedStop::Cancelled);
    assert_original_child(*downstream.unwrap(), PrismPublicationClass::Failed);
}
