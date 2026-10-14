// SPDX-License-Identifier: GPL-2.0-only
use super::{joined_test_support::*, *};
#[test]
fn actual_join_has_complete_exact_raw_endpoint_closure_and_trim_evidence() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let proof = conformance(&join);
    assert!(proof.binds(&join));
    assert_eq!(proof.version(), 2);
    assert_eq!(proof.evidence().placements.len(), 41);
    assert_eq!(proof.evidence().endpoints.len(), 6);
    assert_eq!(proof.evidence().closures.len(), 6);
    assert_eq!(proof.evidence().trims.len(), 6);
    context_counts(&proof);
    exact_bounds(&proof);
}
fn exact_bounds(proof: &JoinedPrismConformance) {
    for row in &proof.evidence().endpoints {
        assert_eq!(row.certificate.proven_squared_upper, zero());
        assert_eq!(row.certificate.requested_spatial, 0.);
        assert_eq!(row.certificate.effective_spatial, 0.);
    }
    for row in &proof.evidence().closures {
        assert_eq!(row.certificate.proven_squared_upper, zero());
    }
    for row in &proof.evidence().trims {
        assert_eq!(row.squared_upper, zero());
    }
}
#[test]
fn metadata_report_and_refuse_retain_all_rows_after_reusing_geometry() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let geometry = conformance(&join);
    let report = tolerance(&join, &geometry, StoredToleranceCap::report(0.).unwrap())
        .ok()
        .unwrap();
    assert!(report.binds(&join));
    assert_eq!(report.metadata().len(), 8);
    assert_eq!(
        report
            .metadata()
            .iter()
            .map(|r| r.contexts.len())
            .sum::<usize>(),
        21
    );
    assert_caps(&join, &geometry);
}
#[test]
fn independently_named_contact_join_cannot_reuse_another_conformance_proof() {
    let mut fixture = JoinFixture::new();
    let join = fixture.compose(|| false).ok().unwrap();
    let proof = conformance(&join);
    fixture.contact.name.0 += 1;
    let renamed = fixture.compose(|| false).ok().unwrap();
    assert!(!proof.binds(&renamed));
    let failure = tolerance(&renamed, &proof, StoredToleranceCap::NoLimit)
        .err()
        .unwrap();
    assert_eq!(failure.classification(), JoinedEvidenceClass::Incomplete);
    assert!(matches!(
        failure.cause,
        JoinedToleranceCause::ConformanceBinding
    ));
    assert!(failure.metadata.is_empty());
}
#[test]
fn exact_policy_keeps_operation_budget_and_no_body_validity_promotion() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let proof = conformance(&join);
    let metadata = tolerance(&join, &proof, StoredToleranceCap::NoLimit)
        .ok()
        .unwrap();
    let expected = join.projection().limits();
    assert_eq!(
        proof.operation_limits().spatial_limit.to_bits(),
        expected.spatial_limit.to_bits()
    );
    assert_eq!(
        metadata.operation_limits().parameter_limit.to_bits(),
        expected.parameter_limit.to_bits()
    );
    assert_eq!(metadata.cap(), StoredToleranceCap::NoLimit);
    assert_eq!(metadata.version(), 2);
    assert!(!join.projection().is_valid_body());
}

fn context_counts(proof: &JoinedPrismConformance) {
    assert_eq!(
        proof
            .evidence()
            .endpoints
            .iter()
            .map(|r| r.contexts.len())
            .sum::<usize>(),
        12
    );
    assert_eq!(
        proof
            .evidence()
            .closures
            .iter()
            .map(|r| r.orientations.len())
            .sum::<usize>(),
        12
    );
}

fn assert_caps(join: &JoinedPrismProof, geometry: &JoinedPrismConformance) {
    let refusal = tolerance(join, geometry, StoredToleranceCap::refuse(0.).unwrap())
        .err()
        .unwrap();
    assert_eq!(refusal.classification(), JoinedEvidenceClass::Failed);
    assert_eq!(refusal.metadata.len(), 8);
    assert!(matches!(
        refusal.cause,
        JoinedToleranceCause::RefusedCap { .. }
    ));
    assert!(tolerance(join, geometry, StoredToleranceCap::NoLimit).is_ok());
}
