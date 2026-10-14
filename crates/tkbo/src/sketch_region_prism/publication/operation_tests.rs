// SPDX-License-Identifier: GPL-2.0-only
use super::{fixture::*, progress::*, result::*, stage_error::*, *};
use crate::sketch_region_partition_bindings::binding_sources;

#[test]
fn complete_operation_owns_same_body_checks_history_and_signed_measures() {
    fn immutable<T: Send + Sync>() {}
    immutable::<PublishedNamedPrism>();
    let fixture = PublicationFixture::new();
    let mut observer = PublicationObserver::default();
    let result = fixture.publish(&mut observer);
    assert_eq!(
        result.completion(),
        PrismPublicationCompletion::CompleteNamedPrismV1
    );
    assert!(result.validation().binds(result.join()));
    assert!(result.history().binds(result.join()));
    assert!(result.measures().matches_join(result.join()));
    assert_eq!(result.requested(), fixture.requested);
    assert!(!result.join().projection().is_valid_body());
    assert_eq!(result.history().continued().len(), 3);
    assert!(result.history().deleted_originals().is_empty());
    assert_eq!(
        observer.seen.first().unwrap().phase,
        PrismPublicationPhase::Join
    );
    assert_eq!(
        observer.seen.last().unwrap().phase,
        PrismPublicationPhase::Commit
    );
    assert_independent_scalars(&result);
}
fn assert_independent_scalars(result: &PublishedNamedPrism) {
    let volume = result.signed_sweep_integral().enclosure();
    assert_eq!(
        volume,
        [
            f64::from_bits(0x4013333333333333),
            f64::from_bits(0x4013333333333334)
        ]
    );
    let cap = [
        f64::from_bits(0x4003333333333333),
        f64::from_bits(0x4003333333333334),
    ];
    for face in result.measures().named_faces() {
        let area = result.measures().face_area(&face.occurrence).unwrap();
        if face.role != crate::sketch_region_prism::PrismImageRole::Generated {
            assert_eq!(area.scalar.enclosure(), cap);
        }
    }
    assert_side_and_total_oracle(result);
    assert!(!result.replay_bytes().is_empty());
}
#[test]
fn unsupported_publication_version_has_no_callback_or_body() {
    let mut fixture = PublicationFixture::new();
    fixture.requested.version = 17;
    let sources = binding_sources(&fixture.operation.shapes);
    let mut observer = PublicationObserver::default();
    let failure = publish_named_prism(&fixture.request(&sources), &mut observer)
        .err()
        .unwrap();
    assert_eq!(failure.operation, fixture.operation.authored.operation);
    assert_eq!(failure.requested, fixture.requested);
    assert_eq!(failure.class, PrismPublicationClass::Incomplete);
    assert!(matches!(
        failure.cause,
        PrismPublicationCause::UnsupportedVersion {
            actual: 17,
            expected: 1
        }
    ));
    assert!(observer.seen.is_empty());
}
#[test]
fn missing_history_names_cannot_publish_or_reach_commit() {
    let mut fixture = PublicationFixture::new();
    fixture.historical.intermediate.clear();
    let sources = binding_sources(&fixture.operation.shapes);
    let mut observer = PublicationObserver::default();
    let failure = publish_named_prism(&fixture.request(&sources), &mut observer)
        .err()
        .unwrap();
    assert_eq!(failure.phase, PrismPublicationPhase::History);
    assert_eq!(failure.class, PrismPublicationClass::Incomplete);
    assert!(matches!(failure.cause, PrismPublicationCause::History(_)));
    assert_eq!(
        observer.seen.last().unwrap().phase,
        PrismPublicationPhase::History
    );
}
#[test]
fn every_stage_cancellation_returns_no_body_and_allows_fresh_reuse() {
    let fixture = PublicationFixture::new();
    let baseline = fixture.publish(&mut PublicationObserver::default());
    let sources = binding_sources(&fixture.operation.shapes);
    for phase in [
        PrismPublicationPhase::Join,
        PrismPublicationPhase::Validation,
        PrismPublicationPhase::History,
        PrismPublicationPhase::Measures,
        PrismPublicationPhase::Commit,
    ] {
        let mut observer = PublicationObserver {
            stop: Some(phase),
            ..Default::default()
        };
        let failure = publish_named_prism(&fixture.request(&sources), &mut observer)
            .err()
            .unwrap();
        assert_eq!(failure.class, PrismPublicationClass::Cancelled);
        assert_eq!(failure.phase, phase);
        assert_eq!(observer.seen.last().unwrap().phase, phase);
        assert_eq!(
            fixture
                .publish(&mut PublicationObserver::default())
                .replay_bytes(),
            baseline.replay_bytes()
        );
    }
}

fn assert_side_and_total_oracle(result: &PublishedNamedPrism) {
    use crate::sketch_region_prism::region_certificate::measurements::tests::complete_tests::{
        assert_contains_interval, oracle_interval,
    };
    let side = result
        .measures()
        .named_faces()
        .iter()
        .find(|face| face.role == crate::sketch_region_prism::PrismImageRole::Generated)
        .unwrap();
    let area = result.measures().face_area(&side.occurrence).unwrap();
    assert_contains_interval(area.scalar.enclosure(), oracle_interval("side"));
    assert_contains_interval(
        result.measures().total_area().enclosure(),
        oracle_interval("total"),
    );
}
