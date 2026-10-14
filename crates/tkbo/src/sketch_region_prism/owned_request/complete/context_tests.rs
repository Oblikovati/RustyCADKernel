// SPDX-License-Identifier: GPL-2.0-only
use super::super::operation_tests::{CompleteObserver, native_failure};
use super::*;
#[test]
fn native_stop_is_reused_by_child_adapters_without_another_callback() {
    let mut observer = CompleteObserver {
        fail: Some(0),
        ..Default::default()
    };
    let mut context = CompletePrismContext::new(&mut observer);
    assert_native_child_reuse(&mut context);
    assert_eq!(
        context.stop.as_ref().unwrap().checkpoint.child_ordinal,
        Some(0)
    );
    assert_eq!(observer.points.len(), 1);
}
fn assert_native_child_reuse(context: &mut CompletePrismContext<'_>) {
    let point = OwnedPrismCheckpoint {
        phase: OwnedPrismPhase::Admission,
        ordinal: 0,
    };
    assert_eq!(
        OwnedPrismProgress::checkpoint(context, point),
        Err(native_failure())
    );
    let later = PrismPublicationCheckpoint {
        phase: PrismPublicationPhase::Commit,
        ordinal: 99,
    };
    assert_eq!(
        PrismPublicationProgress::checkpoint(context, later),
        Err(native_failure())
    );
}
#[test]
fn ordinal_exhaustion_refuses_without_invoking_user_progress() {
    let mut observer = CompleteObserver::default();
    let mut context = CompletePrismContext::new(&mut observer);
    context.ordinal = usize::MAX;
    assert!(context.poll(CompletePrismPhase::Resolution, None));
    assert!(matches!(
        context.stop.as_ref().unwrap().cause,
        CompletePrismCallbackCause::Exhausted
    ));
    assert!(context.poll(CompletePrismPhase::Commit, None));
    assert_eq!(context.stop.unwrap().checkpoint.ordinal, usize::MAX);
    assert!(observer.points.is_empty());
}
