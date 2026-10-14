// SPDX-License-Identifier: GPL-2.0-only
//! Callback class/header provenance is structural; no class is inferred from its cause.
use super::*;
fn callback_stop() -> Box<PublicationCallbackStop> {
    Box::new(PublicationCallbackStop {
        checkpoint: PrismPublicationCheckpoint {
            phase: PrismPublicationPhase::Measures,
            ordinal: 7,
        },
        cause: PublicationCallbackCause::Exhausted,
    })
}
#[test]
fn v2_absent_child_keeps_exact_exhaustion_checkpoint_without_gap() {
    with_diagnostic_context(|bindings, options| {
        let cause = PublicationStageCause::CallbackWithChild {
            stop: callback_stop(),
            downstream: None,
        };
        let mut gaps = Vec::new();
        let result = projection::outer(&outer(cause, options), bindings, &mut gaps);
        let OwnedPublicationStageCause::CallbackV2 { stop, downstream } = &stage(&result).cause
        else {
            panic!("expected V2")
        };
        assert!(gaps.is_empty());
        assert!(downstream.is_none());
        assert_eq!(stop.checkpoint.ordinal, 7);
        assert_eq!(stop.checkpoint.phase, PrismPublicationPhase::Measures);
        assert_eq!(stop.cause, OwnedDiagnosticCallbackCause::Exhausted);
    });
}
#[test]
fn v2_binding_child_preserves_class_unrelated_to_outer_callback() {
    with_diagnostic_context(|bindings, options| {
        for class in [
            PrismPublicationClass::Failed,
            PrismPublicationClass::Incomplete,
            PrismPublicationClass::Cancelled,
        ] {
            assert_child(bindings, options, class);
        }
    });
}
fn assert_child(
    bindings: &PrismPublicationBindings,
    options: PrismPublicationOptions,
    class: PrismPublicationClass,
) {
    let cause = binding_child(class);
    let mut gaps = Vec::new();
    let result = projection::outer(&outer(cause, options), bindings, &mut gaps);
    let OwnedPublicationStageCause::CallbackV2 {
        downstream: Some(child),
        ..
    } = &stage(&result).cause
    else {
        panic!("expected child")
    };
    assert!(gaps.is_empty());
    assert_child_payload(child, class);
}
fn binding_child(class: PrismPublicationClass) -> PublicationStageCause {
    let child = PublicationStageFailure {
        phase: PrismPublicationPhase::Commit,
        class,
        cause: PublicationStageCause::Binding {
            proof: "exact original proof",
        },
    };
    PublicationStageCause::CallbackWithChild {
        stop: callback_stop(),
        downstream: Some(Box::new(child)),
    }
}
fn assert_child_payload(child: &OwnedPublicationStageDiagnostic, class: PrismPublicationClass) {
    assert_eq!(child.class, class);
    assert_eq!(child.phase, PrismPublicationPhase::Commit);
    assert!(matches!(
        child.cause,
        OwnedPublicationStageCause::Binding {
            proof: "exact original proof"
        }
    ));
}
