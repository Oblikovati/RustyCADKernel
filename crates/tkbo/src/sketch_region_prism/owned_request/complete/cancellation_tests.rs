// SPDX-License-Identifier: GPL-2.0-only
//! Every prefix uses the full owned entry again; no cached prepared fixture bypasses composition.
use super::super::{OwnedPolynomialPrismRequest, request_replay::encode_owned_prism_request};
use super::{operation_tests::*, *};
#[test]
fn every_complete_checkpoint_cancel_and_native_failure_is_terminal_and_reusable() {
    let (accepted, baseline) = accepted();
    let request = accepted.request();
    for point in &baseline.points {
        for failed in [false, true] {
            terminal_prefix(request, &baseline.points, point.ordinal, failed);
        }
    }
    let mut progress = CompleteObserver::default();
    with_owned_prism_operation(request.clone(), &mut progress, |result| {
        let repeated = result.unwrap_or_else(|_| panic!("fresh reuse required"));
        assert_eq!(repeated.replay_bytes(), accepted.replay_bytes());
    });
    println!("SCOPED_COMPLETE_CHECKPOINTS={}", baseline.points.len());
}
fn terminal_prefix(
    request: &OwnedPolynomialPrismRequest,
    baseline: &[CompletePrismCheckpoint],
    ordinal: usize,
    failed: bool,
) {
    let mut observer = CompleteObserver {
        stop: (!failed).then_some(ordinal),
        fail: failed.then_some(ordinal),
        ..Default::default()
    };
    with_owned_prism_operation(request.clone(), &mut observer, |result| {
        inspect_terminal_result(result, request, baseline[ordinal], failed);
    });
    assert_eq!(observer.points, baseline[..=ordinal]);
}
fn assert_stop(failure: &ScopedPrismFailure<'_>, point: CompletePrismCheckpoint, failed: bool) {
    let stopped = failure.callback().expect("exact initiating outer callback");
    assert_eq!(stopped.checkpoint, point);
    match &stopped.cause {
        CompletePrismCallbackCause::Failed(cause) if failed => assert_eq!(*cause, native_failure()),
        CompletePrismCallbackCause::Cancelled if !failed => (),
        _ => panic!("outer stop must preserve actual native/cancel cause"),
    }
}
fn assert_local_cause(failure: ScopedPrismFailure<'_>, failed: bool) {
    let (_, cause) = failure.into_parts();
    match cause {
        ScopedPrismCause::Preparation(original) => {
            assert_eq!(original.class, preparation_class(failed));
            assert!(original.callback.is_some());
        }
        ScopedPrismCause::Resolution { prepared, .. } => {
            assert!(prepared.retained.source.is_some())
        }
        ScopedPrismCause::Publication(original) => {
            assert_eq!(original.failure().class, publication_class(failed))
        }
        ScopedPrismCause::Projection(original) => {
            assert!(original.invoked.result().replay_bytes().len() > 1_000_000)
        }
    }
}
fn preparation_class(failed: bool) -> super::super::OwnedPreparationClass {
    if failed {
        return super::super::OwnedPreparationClass::Failed;
    }
    super::super::OwnedPreparationClass::Cancelled
}
fn publication_class(
    failed: bool,
) -> crate::sketch_region_prism::publication::PrismPublicationClass {
    use crate::sketch_region_prism::publication::PrismPublicationClass::*;
    if failed {
        return Failed;
    }
    Cancelled
}

fn inspect_terminal_result(
    result: Result<OwnedCompletePrism, ScopedPrismFailure<'_>>,
    request: &OwnedPolynomialPrismRequest,
    point: CompletePrismCheckpoint,
    failed: bool,
) {
    let failure = result
        .err()
        .unwrap_or_else(|| panic!("checkpoint {} must refuse", point.ordinal));
    assert_eq!(
        encode_owned_prism_request(failure.request()),
        encode_owned_prism_request(request)
    );
    assert_stop(&failure, point, failed);
    assert_local_cause(failure, failed);
}
