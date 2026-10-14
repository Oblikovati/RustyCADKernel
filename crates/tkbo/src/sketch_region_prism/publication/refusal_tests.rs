// SPDX-License-Identifier: GPL-2.0-only
//! Missing policies and typed callback failures cannot escape as accepted geometry.
use super::{fixture::*, progress::*, stage_error::*, *};
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkernel::standard::Failure;
struct FailingPublicationObserver {
    phase: PrismPublicationPhase,
    seen: Vec<PrismPublicationPhase>,
}
impl PrismPublicationProgress for FailingPublicationObserver {
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.seen.push(point.phase);
        if point.phase == self.phase {
            return Err(Failure::out_of_range(
                "publication observer refused checkpoint; expected successful observation",
            ));
        }
        Ok(false)
    }
}
#[test]
fn every_stage_observer_failure_preserves_typed_cause_and_stops_publication() {
    let fixture = PublicationFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    for phase in publication_phases() {
        let mut observer = FailingPublicationObserver {
            phase,
            seen: Vec::new(),
        };
        let failure = publish_named_prism(&fixture.request(&sources), &mut observer)
            .err()
            .unwrap();
        assert_eq!(failure.phase, phase);
        assert_eq!(failure.class, PrismPublicationClass::Failed);
        assert_eq!(observer.seen.last(), Some(&phase));
        assert_native_callback(failure.cause);
    }
}
fn assert_native_callback(cause: PrismPublicationCause<'_>) {
    let PrismPublicationCause::Stage(stage) = cause else {
        panic!("expected stage failure");
    };
    let PublicationStageCause::Callback { stop, .. } = stage.cause else {
        panic!("expected callback failure");
    };
    assert!(matches!(stop.cause, PublicationCallbackCause::Failed(_)));
}
#[test]
fn every_child_version_refuses_incomplete_at_its_stage() {
    let fixture = PublicationFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    for phase in [
        PrismPublicationPhase::Join,
        PrismPublicationPhase::Validation,
        PrismPublicationPhase::History,
        PrismPublicationPhase::Measures,
    ] {
        let mut request = fixture.request(&sources);
        invalidate_version(&mut request.requested, phase);
        let failure = publish_named_prism(&request, &mut PublicationObserver::default())
            .err()
            .unwrap();
        assert_eq!(failure.phase, phase);
        assert_eq!(failure.class, PrismPublicationClass::Incomplete);
    }
}
fn invalidate_version(requested: &mut PrismPublicationOptions, phase: PrismPublicationPhase) {
    match phase {
        PrismPublicationPhase::Join => requested.join.version = 17,
        PrismPublicationPhase::Validation => requested.validation.version = 17,
        PrismPublicationPhase::History => requested.history.version = 17,
        PrismPublicationPhase::Measures => requested.measures.version = 17,
        _ => panic!("expected required child stage, got {phase:?}"),
    }
}
#[test]
fn explicit_zero_tolerance_cap_is_failed_and_cannot_publish_a_partial_body() {
    let fixture = PublicationFixture::new();
    let sources = binding_sources(&fixture.operation.shapes);
    let mut request = fixture.request(&sources);
    request.requested.validation =
        crate::sketch_region_prism::frozen::ordered_validation::OrderedPrismValidationOptions::v1(
            crate::sketch_region_prism::exact_tolerance::StoredToleranceCap::refuse(0.).unwrap(),
        );
    let failure = publish_named_prism(&request, &mut PublicationObserver::default())
        .err()
        .unwrap();
    assert_eq!(failure.phase, PrismPublicationPhase::Validation);
    assert_eq!(failure.class, PrismPublicationClass::Failed);
}
