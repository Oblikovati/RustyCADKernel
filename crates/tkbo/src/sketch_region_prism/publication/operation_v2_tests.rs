// SPDX-License-Identifier: GPL-2.0-only
//! Explicit V2 selection changes diagnostic retention, never the admitted geometry or proof children.
use super::{fixture::*, *};
use crate::sketch_region_partition_bindings::binding_sources;

#[test]
fn v2_success_retains_exact_v1_children_and_labels_its_requested_behavior() {
    let mut fixture = PublicationFixture::new();
    let v1 = fixture.publish(&mut PublicationObserver::default());
    fixture.requested.version = 2;
    let v2 = publish_v2_fixture(&fixture);
    assert_eq!(
        v2.completion(),
        PrismPublicationCompletion::CompleteNamedPrismV2
    );
    assert_eq!(v2.requested().version, 2);
    assert_same_children(&v1, &v2);
    let offset = b"Martensite complete named prism publication V1\n".len();
    let mut expected = v1.replay_bytes().to_vec();
    expected[offset..offset + 4].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(v2.replay_bytes(), expected);
}
fn publish_v2_fixture(fixture: &PublicationFixture) -> PublishedNamedPrism {
    let sources = binding_sources(&fixture.operation.shapes);
    publish_named_prism_v2(
        &fixture.request(&sources),
        &mut PublicationObserver::default(),
    )
    .ok()
    .unwrap()
}
fn assert_same_children(v1: &PublishedNamedPrism, v2: &PublishedNamedPrism) {
    assert_eq!(v1.join().replay_bytes(), v2.join().replay_bytes());
    assert_eq!(
        v1.validation().replay_bytes(),
        v2.validation().replay_bytes()
    );
    assert_eq!(v1.history().replay_bytes(), v2.history().replay_bytes());
    assert_eq!(v1.measures().replay_bytes(), v2.measures().replay_bytes());
}
#[test]
fn entries_refuse_the_other_version_without_callbacks() {
    let mut fixture = PublicationFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    let mut observer = PublicationObserver::default();
    let error = publish_named_prism_v2(&fixture.request(&sources), &mut observer)
        .err()
        .unwrap();
    assert_version_refusal(&error, 1, 2);
    fixture.requested.version = 2;
    let error = publish_named_prism(&fixture.request(&sources), &mut observer)
        .err()
        .unwrap();
    assert_version_refusal(&error, 2, 1);
    assert!(observer.seen.is_empty());
}
fn assert_version_refusal(error: &PrismPublicationFailure<'_>, actual: u32, expected: u32) {
    assert_eq!(error.phase, PrismPublicationPhase::Admission);
    assert_eq!(error.class, PrismPublicationClass::Incomplete);
    assert!(
        matches!(error.cause, PrismPublicationCause::UnsupportedVersion { actual:a, expected:e } if a==actual && e==expected)
    );
}
#[test]
fn actual_v2_callback_retains_child_stage_and_unchanged_outer_class() {
    let mut fixture = PublicationFixture::new();
    fixture.requested.version = 2;
    let sources = binding_sources(&fixture.operation.shapes);
    let mut observer = PublicationObserver {
        stop: Some(PrismPublicationPhase::Validation),
        ..Default::default()
    };
    let failure = publish_named_prism_v2(&fixture.request(&sources), &mut observer)
        .err()
        .unwrap();
    assert_eq!(failure.class, PrismPublicationClass::Cancelled);
    assert_eq!(failure.phase, PrismPublicationPhase::Validation);
    assert_validation_child(failure.cause);
}
fn assert_validation_child(cause: PrismPublicationCause<'_>) {
    let PrismPublicationCause::Stage(stage) = cause else {
        panic!("expected actual validation stage")
    };
    let PublicationStageCause::CallbackWithChild { stop, downstream } = stage.cause else {
        panic!("expected actual V2 callback child")
    };
    assert_eq!(stop.checkpoint.phase, PrismPublicationPhase::Validation);
    assert!(matches!(stop.cause, PublicationCallbackCause::Cancelled));
    let child = downstream.unwrap();
    assert_eq!(child.phase, PrismPublicationPhase::Validation);
    assert_eq!(child.class, PrismPublicationClass::Cancelled);
    assert!(matches!(child.cause, PublicationStageCause::Validation(_)));
}
fn independent_v2_bytes() -> Vec<u8> {
    let mut fixture = PublicationFixture::new();
    fixture.requested.version = 2;
    publish_v2_fixture(&fixture).replay_bytes().to_vec()
}
#[test]
fn v2_publication_process_replay() {
    let bytes = independent_v2_bytes();
    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    println!("named-prism-publication-v2={hex}");
}
#[test]
fn v2_independent_threads_match_serial_complete_result() {
    static_assertions::assert_impl_all!(PrismPublicationCompletion: Send, Sync);
    let expected = independent_v2_bytes();
    let workers: Vec<_> = (0..4)
        .map(|_| std::thread::spawn(independent_v2_bytes))
        .collect();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), expected);
    }
    assert_eq!(independent_v2_bytes(), expected);
}
