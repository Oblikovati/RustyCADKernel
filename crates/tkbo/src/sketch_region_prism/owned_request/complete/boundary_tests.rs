// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    OwnedPreparationCause, OwnedPreparationClass, OwnedPrismAdmissionCause,
    resolver::ResolutionCause, test_fixture,
};
use super::{operation_tests::*, *};
use crate::sketch_region_prism::{
    publication::*, region_certificate::current_binding::PrismBindingFailure,
};
#[test]
fn empty_input_declines_without_callback_or_fabricated_names() {
    let mut request = test_fixture::request();
    request.sources.clear();
    let mut progress = CompleteObserver::default();
    with_owned_prism_operation(request, &mut progress, |result| {
        let failure = result.err().unwrap();
        assert!(failure.callback().is_none());
        let ScopedPrismCause::Preparation(original) = failure.into_parts().1 else {
            panic!("admission")
        };
        assert_eq!(original.class, OwnedPreparationClass::Declined);
        assert!(matches!(
            original.cause,
            OwnedPreparationCause::Admission(OwnedPrismAdmissionCause::EmptyInput)
        ));
        assert!(original.retained.plan.is_none());
    });
    assert!(progress.points.is_empty());
}
#[test]
fn multiple_input_is_explicit_incomplete_domain_with_all_inputs_retained() {
    let mut request = test_fixture::request();
    request.sources.push(request.sources[0].clone());
    let mut progress = CompleteObserver::default();
    with_owned_prism_operation(request, &mut progress, |result| {
        let failure = result.err().unwrap();
        assert_multiple(failure);
    });
    assert!(progress.points.is_empty());
}
#[test]
fn resolver_bool_callback_keeps_outer_native_failure_and_original_child_cancel() {
    let (accepted, baseline) = accepted();
    let point = *baseline
        .points
        .iter()
        .find(|p| p.phase == CompletePrismPhase::Resolution)
        .unwrap();
    let mut progress = CompleteObserver {
        fail: Some(point.ordinal),
        ..Default::default()
    };
    with_owned_prism_operation(accepted.request().clone(), &mut progress, |result| {
        let failure = result.err().unwrap();
        assert_resolver_stop(failure, point);
    });
    assert_eq!(progress.points, baseline.points[..=point.ordinal]);
}
#[test]
fn final_commit_refusal_retains_the_exact_accepted_private_publication() {
    let (accepted, baseline) = accepted();
    let point = *baseline.points.last().unwrap();
    let mut progress = CompleteObserver {
        stop: Some(point.ordinal),
        ..Default::default()
    };
    with_owned_prism_operation(accepted.request().clone(), &mut progress, |result| {
        let failure = result.err().unwrap();
        assert_final_stop(failure, point, &accepted);
    });
    assert_eq!(progress.points, baseline.points);
}
#[test]
fn unsupported_publication_version_retains_exact_invoked_packet_and_header() {
    let mut request = test_fixture::request();
    request.policy.publication.version = 17;
    let mut progress = CompleteObserver::default();
    with_owned_prism_operation(request, &mut progress, |result| {
        let failure = result.err().unwrap();
        assert_version_packet(failure);
    });
}

fn assert_multiple(failure: ScopedPrismFailure<'_>) {
    assert_eq!(failure.request().sources.len(), 2);
    let ScopedPrismCause::Preparation(original) = failure.into_parts().1 else {
        panic!("admission")
    };
    assert_eq!(original.class, OwnedPreparationClass::Incomplete);
    assert!(matches!(
        original.cause,
        OwnedPreparationCause::Admission(OwnedPrismAdmissionCause::MultipleBoundaries {
            supported: 1,
            ..
        })
    ));
}

fn assert_resolver_stop(failure: ScopedPrismFailure<'_>, point: CompletePrismCheckpoint) {
    assert_eq!(failure.callback().unwrap().checkpoint, point);
    assert!(
        matches!(&failure.callback().unwrap().cause, CompletePrismCallbackCause::Failed(c) if *c == native_failure())
    );
    let ScopedPrismCause::Resolution { cause, prepared } = failure.into_parts().1 else {
        panic!("resolver")
    };
    assert_eq!(prepared.retained.plan.as_ref().unwrap().version(), 2);
    assert!(
        matches!(*cause, ResolutionCause::Binding(c) if matches!(*c, PrismBindingFailure::Cancelled))
    );
}

fn assert_final_stop(
    failure: ScopedPrismFailure<'_>,
    point: CompletePrismCheckpoint,
    accepted: &OwnedCompletePrism,
) {
    assert_eq!(failure.callback().unwrap().checkpoint, point);
    let ScopedPrismCause::Projection(original) = failure.into_parts().1 else {
        panic!("projection")
    };
    assert!(matches!(
        original.cause,
        super::projection::CompleteProjectionCause::Callback
    ));
    assert_eq!(
        original.invoked.result().replay_bytes(),
        accepted.publication().replay_bytes()
    );
}

fn assert_version_packet(failure: ScopedPrismFailure<'_>) {
    assert!(failure.callback().is_none());
    let ScopedPrismCause::Publication(packet) = failure.into_parts().1 else {
        panic!("publication")
    };
    assert_eq!(packet.request().policy.publication.version, 17);
    assert_eq!(packet.plan().version(), 2);
    let original = packet.into_failure();
    assert_eq!(original.class, PrismPublicationClass::Incomplete);
    assert!(matches!(
        original.cause,
        PrismPublicationCause::UnsupportedVersion {
            actual: 17,
            expected: 1
        }
    ));
}
