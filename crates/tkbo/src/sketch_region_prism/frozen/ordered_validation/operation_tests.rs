// SPDX-License-Identifier: GPL-2.0-only
use super::{test_support::*, *};
use crate::sketch_region_validation::StageCoverage;

#[test]
fn complete_operation_uses_one_join_and_retains_historical_failed_checker() {
    let fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    let source = join.replay_bytes().to_vec();
    let proof = complete(&join);
    for stage in OrderedPrismStage::ALL {
        assert_eq!(proof.trace().status(stage), OrderedPrismStageStatus::Passed);
    }
    assert!(proof.binds(&join));
    assert_eq!(
        proof.mode(),
        OrderedPrismValidationMode::ExactSingleJordanPolynomialPrism
    );
    assert_eq!(proof.requested(), options());
    assert_eq!(join.replay_bytes(), source);
    assert!(!join.projection().is_valid_body());
    assert!(fixture.rebind(&join).is_ok());
    check_proofs(&join, &proof);
    check_history(&proof);
}
fn check_proofs(join: &JoinedPrismProof, proof: &OrderedPrismValidationCertificate) {
    assert!(proof.topology().binds(join.projection()));
    assert_eq!(
        proof.topology().achieved_error(),
        topology::TopologyNumericalError::NotApplicable
    );
    assert!(proof.orientation().matches_join(join));
    assert_eq!(proof.orientation().faces().len(), 3);
    assert!(proof.conformance().binds(join));
    assert_eq!(proof.conformance().evidence().endpoints.len(), 6);
    assert_eq!(proof.conformance().evidence().closures.len(), 6);
    assert_eq!(proof.conformance().evidence().trims.len(), 6);
    check_intersection(join, proof);
    assert!(proof.tolerance().binds(join));
    assert_eq!(proof.tolerance().metadata().len(), 8);
    assert_eq!(proof.tolerance().cap(), options().stored_cap);
}
fn check_intersection(join: &JoinedPrismProof, proof: &OrderedPrismValidationCertificate) {
    assert!(proof.intersection().binds(join));
    assert_eq!(proof.intersection().domains(), join.embedding().domains());
    assert_eq!(
        proof.intersection().incidences(),
        join.embedding().incidences()
    );
}
fn check_history(proof: &OrderedPrismValidationCertificate) {
    use super::super::{
        alerts::FrozenNativeAlert::IntersectionFailedAtFailGravity,
        coverage::FrozenValidationCause::AnalyticNativeAlert,
    };
    use StageCoverage::{Blocked, Incomplete, Passed};
    assert_eq!(
        proof.historical().historical_unbound,
        [Passed, Passed, Incomplete, Blocked]
    );
    assert_eq!(proof.historical().current, [Incomplete; 4]);
    assert!(proof.historical().findings.iter().any(|row| matches!(
        row.cause,
        AnalyticNativeAlert(IntersectionFailedAtFailGravity)
    )));
}
#[test]
fn unknown_current_behavior_is_incomplete_before_any_geometric_work() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut requested = options();
    requested.version = 99;
    let mut progress = NamedValidationProgress::default();
    let refused = failure(validate_joined_prism(&join, requested, &mut progress));
    assert!(matches!(
        refused.cause,
        OrderedPrismCause::Version {
            actual: 99,
            expected: 1
        }
    ));
    assert_eq!(refused.class, OrderedPrismFailureClass::Incomplete);
    assert!(progress.points.is_empty());
    assert_eq!(
        refused.trace.status(OrderedPrismStage::Geometry),
        OrderedPrismStageStatus::Blocked
    );
}
#[test]
fn actual_metadata_refusal_occurs_only_after_completed_geometry_and_si() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let request = OrderedPrismValidationOptions::v1(StoredToleranceCap::refuse(0.).unwrap());
    let refused = failure(validate_joined_prism(
        &join,
        request,
        &mut NamedValidationProgress::default(),
    ));
    assert_eq!(refused.class, OrderedPrismFailureClass::Failed);
    assert_eq!(refused.checkpoint.stage, OrderedPrismStage::Tolerance);
    check_completed_prefix(&refused);
    let OrderedPrismCause::Tolerance(cause) = refused.cause else {
        panic!("expected cap refusal")
    };
    assert_eq!(cause.metadata.len(), 8);
    assert!(matches!(
        cause.cause,
        crate::sketch_region_prism::exact_tolerance::JoinedToleranceCause::RefusedCap { .. }
    ));
}
#[test]
fn immutable_validation_types_and_failure_payloads_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<OrderedPrismValidationOptions>();
    assert_send_sync::<OrderedPrismValidationCertificate>();
    assert_send_sync::<OrderedPrismValidationFailure>();
    assert_send_sync::<OrderedPrismValidationOutcome>();
    assert_send_sync::<OrderedPrismCause>();
    assert_send_sync::<OrderedPrismCheckpoint>();
    assert_send_sync::<OrderedPrismCheck>();
    assert_send_sync::<OrderedPrismValidationMode>();
    assert_send_sync::<JoinedPrismIntersectionCertificate>();
    assert_send_sync::<OrderedPrismFailureClass>();
}

fn check_completed_prefix(refused: &OrderedPrismValidationFailure) {
    for stage in [
        OrderedPrismStage::Topology,
        OrderedPrismStage::Geometry,
        OrderedPrismStage::SelfIntersection,
    ] {
        assert_eq!(refused.trace.status(stage), OrderedPrismStageStatus::Passed);
    }
}
