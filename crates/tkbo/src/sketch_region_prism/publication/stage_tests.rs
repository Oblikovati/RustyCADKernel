// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::{
    exact_tolerance::StoredToleranceCap,
    frozen::proof_join::operation_tests::JoinFixture,
    region_certificate::{CompletePrismMeasureOptions, PrismMeasureOptions},
};
use rustycad_tkmath::predicates::roots::metric_bounds::{
    EnclosureSumOptions, PolynomialSideOptions,
};

#[derive(Default)]
struct NamedStageProgress {
    stop: bool,
    fail: bool,
    seen: Vec<PrismPublicationPhase>,
}
impl PrismPublicationProgress for NamedStageProgress {
    fn checkpoint(&mut self, point: PrismPublicationCheckpoint) -> Result<bool, Failure> {
        self.seen.push(point.phase);
        if self.fail {
            return Err(Failure::new(
                "publication callback failed: expected successful stage",
            ));
        }
        Ok(self.stop)
    }
}
fn validation_options() -> OrderedPrismValidationOptions {
    OrderedPrismValidationOptions::v1(StoredToleranceCap::refuse(1e-6).unwrap())
}
fn measure_options() -> JoinedPrismMeasureOptions {
    JoinedPrismMeasureOptions::v1(CompletePrismMeasureOptions {
        subset: PrismMeasureOptions {
            cap_area: Default::default(),
            signed_volume: Default::default(),
        },
        side_area: PolynomialSideOptions {
            absolute_error_limit: 0.01,
            ..Default::default()
        },
        total_area: EnclosureSumOptions {
            absolute_error_limit: 0.02,
            ..Default::default()
        },
    })
}
#[test]
fn actual_validation_and_measures_share_bytes_without_publishing_a_body() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut progress = NamedStageProgress::default();
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    let checked = validation(&join, validation_options(), &mut callbacks)
        .ok()
        .unwrap();
    let measured = measures(&join, measure_options(), &mut callbacks)
        .ok()
        .unwrap();
    assert!(checked.binds(&join));
    assert!(measured.matches_join(&join));
    assert!(measured.signed_volume().enclosure()[0] <= 4.8);
    assert!(measured.signed_volume().enclosure()[1] >= 4.8);
    assert!(!join.projection().is_valid_body());
    assert_eq!(progress.seen[0], PrismPublicationPhase::Validation);
    assert_eq!(progress.seen.last(), Some(&PrismPublicationPhase::Measures));
}
#[test]
fn invalid_validation_version_remains_incomplete() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut progress = NamedStageProgress::default();
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    let mut requested = validation_options();
    requested.version = 17;
    let failure = validation(&join, requested, &mut callbacks).err().unwrap();
    assert_eq!(failure.class, PrismPublicationClass::Incomplete);
    assert!(progress.seen.is_empty());
}
#[test]
fn invalid_measure_version_remains_incomplete() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut progress = NamedStageProgress::default();
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    let mut requested = measure_options();
    requested.version = 17;
    let failure = measures(&join, requested, &mut callbacks).err().unwrap();
    assert_eq!(failure.class, PrismPublicationClass::Incomplete);
    assert!(progress.seen.is_empty());
}
#[test]
fn real_cap_refusal_is_failed_and_preserves_child_evidence() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut progress = NamedStageProgress::default();
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    let requested = OrderedPrismValidationOptions::v1(StoredToleranceCap::refuse(0.).unwrap());
    let failure = validation(&join, requested, &mut callbacks).err().unwrap();
    assert_eq!(failure.class, PrismPublicationClass::Failed);
    assert!(matches!(
        failure.cause,
        PublicationStageCause::Validation(_)
    ));
}
#[test]
fn callback_failure_retains_original_and_downstream_refusal() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut progress = NamedStageProgress {
        fail: true,
        ..Default::default()
    };
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    let failure = validation(&join, validation_options(), &mut callbacks)
        .err()
        .unwrap();
    assert_eq!(failure.class, PrismPublicationClass::Failed);
    let PublicationStageCause::Callback { stop, downstream } = failure.cause else {
        panic!("expected retained callback failure");
    };
    assert!(matches!(stop.cause, PublicationCallbackCause::Failed(_)));
    assert!(downstream.is_some());
    assert_eq!(progress.seen.len(), 1);
}
#[test]
fn scalar_callback_cancellation_keeps_partial_cause_without_result() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut progress = NamedStageProgress {
        stop: true,
        ..Default::default()
    };
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    let failure = measures(&join, measure_options(), &mut callbacks)
        .err()
        .unwrap();
    assert_eq!(failure.class, PrismPublicationClass::Cancelled);
    assert!(matches!(
        failure.cause,
        PublicationStageCause::Callback {
            downstream: Some(_),
            ..
        }
    ));
    assert_eq!(progress.seen.len(), 1);
}

#[test]
fn child_proofs_from_another_authored_operation_are_incomplete() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut progress = NamedStageProgress::default();
    let mut callbacks = PublicationCallbacks::new(&mut progress);
    let validated = validation(&join, validation_options(), &mut callbacks)
        .ok()
        .unwrap();
    let measured = measures(&join, measure_options(), &mut callbacks)
        .ok()
        .unwrap();
    let other = differently_named_join();
    let validation = bound_validation(&other, validated).err().unwrap();
    let measures = bound_measures(&other, measured).err().unwrap();
    assert_eq!(validation.class, PrismPublicationClass::Incomplete);
    assert_eq!(measures.class, PrismPublicationClass::Incomplete);
}
fn differently_named_join() -> JoinedPrismProof {
    let mut fixture = JoinFixture::new();
    fixture.authored.operation = crate::sketch_region_prism::frozen::AuthoredPrismId(800);
    fixture.compose(|| false).ok().unwrap()
}
