// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_prism::{
    exact_tolerance::StoredToleranceCap, frozen::annular_join::operation_tests::joined,
};
struct Continue;
impl AnnularConformanceProgress for Continue {
    fn cancelled(&mut self, _: JoinedConformancePhase) -> bool {
        false
    }
}
impl AnnularToleranceProgress for Continue {
    fn cancelled(&mut self, _: JoinedTolerancePhase) -> bool {
        false
    }
}
fn conformance(join: &AnnularFrozenDomain) -> AnnularPrismConformance {
    certify_annular_prism_conformance(join, &mut Continue).unwrap_or_else(|failure| {
        panic!(
            "annular geometry {:?}/{:?}",
            failure.phase,
            failure.classification()
        )
    })
}
#[test]
fn actual_annular_conformance_covers_both_loops_and_keeps_body_incomplete() {
    let joined = joined();
    let proof = conformance(&joined);
    assert!(proof.binds(&joined));
    assert_eq!(proof.evidence().body.endpoints.len(), 12);
    assert_eq!(
        proof
            .evidence()
            .body
            .endpoints
            .iter()
            .map(|row| row.contexts.len())
            .sum::<usize>(),
        24
    );
    assert_eq!(proof.evidence().body.trims.len(), 12);
    assert_eq!(proof.evidence().closures.len(), 20);
    assert_eq!(proof.evidence().source_reports.len(), 6);
    assert_eq!(proof.evidence().supports.len(), 10);
    assert!(!joined.projection().is_valid_body());
    assert_eq!(
        proof.operation_limits().spatial_limit.to_bits(),
        joined.projection().limits().spatial_limit.to_bits()
    );
}
#[test]
fn actual_annular_metadata_reuses_geometry_and_keeps_all_refused_observations() {
    let joined = joined();
    let proof = conformance(&joined);
    let accepted = certify_annular_prism_tolerances(
        &joined,
        &proof,
        StoredToleranceCap::NoLimit,
        &mut Continue,
    )
    .ok()
    .unwrap();
    assert_eq!(accepted.metadata().len(), 14);
    assert!(accepted.binds(&joined));
    let refused = certify_annular_prism_tolerances(
        &joined,
        &proof,
        StoredToleranceCap::refuse(0.).unwrap(),
        &mut Continue,
    )
    .err()
    .unwrap();
    assert_eq!(refused.metadata.len(), 14);
    assert_eq!(
        refused.classification(),
        super::super::JoinedEvidenceClass::Failed
    );
    assert!(matches!(
        refused.cause,
        JoinedToleranceCause::RefusedCap { .. }
    ));
}
#[test]
fn owned_annular_proofs_and_all_failures_are_send_sync() {
    fn assert_owned<T: Send + Sync>() {}
    assert_owned::<AnnularPrismConformance>();
    assert_owned::<AnnularPrismTolerance>();
    assert_owned::<AnnularConformanceFailure>();
    assert_owned::<AnnularConformanceCause>();
    assert_owned::<AnnularToleranceFailure>();
    assert_owned::<AnnularConformanceEvidence>();
}
