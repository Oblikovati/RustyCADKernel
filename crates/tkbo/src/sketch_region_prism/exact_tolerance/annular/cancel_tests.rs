// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region_prism::{
    exact_tolerance::{JoinedEvidenceClass, StoredToleranceCap},
    frozen::annular_join::operation_tests::joined,
};
#[derive(Default)]
struct StopGeometry {
    cutoff: Option<usize>,
    calls: Vec<JoinedConformancePhase>,
}
impl AnnularConformanceProgress for StopGeometry {
    fn cancelled(&mut self, phase: JoinedConformancePhase) -> bool {
        let index = self.calls.len();
        self.calls.push(phase);
        self.cutoff == Some(index)
    }
}
#[derive(Default)]
struct StopMetadata {
    cutoff: Option<usize>,
    calls: Vec<JoinedTolerancePhase>,
}
impl AnnularToleranceProgress for StopMetadata {
    fn cancelled(&mut self, phase: JoinedTolerancePhase) -> bool {
        let index = self.calls.len();
        self.calls.push(phase);
        self.cutoff == Some(index)
    }
}
#[test]
fn every_annular_geometry_checkpoint_cancels_with_exact_prefix_and_reuse() {
    let join = joined();
    let mut baseline = StopGeometry::default();
    let expected = certify_annular_prism_conformance(&join, &mut baseline)
        .ok()
        .unwrap()
        .replay_bytes();
    cancel_geometry_prefixes(&join, &baseline);
    assert_eq!(conformance(&join).replay_bytes(), expected);
    println!(
        "annular-conformance-cancellation-checkpoints={}",
        baseline.calls.len()
    );
}
#[test]
fn every_annular_metadata_checkpoint_cancels_without_losing_prior_observations() {
    let join = joined();
    let proof = conformance(&join);
    let mut baseline = StopMetadata::default();
    let expected =
        certify_annular_prism_tolerances(&join, &proof, StoredToleranceCap::NoLimit, &mut baseline)
            .ok()
            .unwrap()
            .replay_bytes();
    assert_eq!(baseline.calls.len(), 16);
    cancel_metadata_prefixes(&join, &proof, &baseline);
    assert_metadata_reuse(&join, &proof, &expected);
}

fn cancel_geometry_prefixes(join: &AnnularFrozenDomain, baseline: &StopGeometry) {
    for cutoff in 0..baseline.calls.len() {
        let mut stop = StopGeometry {
            cutoff: Some(cutoff),
            calls: Vec::new(),
        };
        let failure = certify_annular_prism_conformance(join, &mut stop)
            .err()
            .unwrap();
        assert_eq!(failure.classification(), JoinedEvidenceClass::Cancelled);
        assert_eq!(failure.phase, baseline.calls[cutoff]);
        assert_eq!(stop.calls, baseline.calls[..=cutoff]);
        assert_eq!(failure.binding, join.replay_bytes());
    }
}

fn cancel_metadata_prefixes(
    join: &AnnularFrozenDomain,
    proof: &AnnularPrismConformance,
    baseline: &StopMetadata,
) {
    for cutoff in 0..baseline.calls.len() {
        let mut stop = StopMetadata {
            cutoff: Some(cutoff),
            calls: Vec::new(),
        };
        let failure =
            certify_annular_prism_tolerances(join, proof, StoredToleranceCap::NoLimit, &mut stop)
                .err()
                .unwrap();
        assert_eq!(failure.classification(), JoinedEvidenceClass::Cancelled);
        assert_eq!(failure.metadata.len(), cutoff.saturating_sub(1).min(14));
        assert_eq!(stop.calls, baseline.calls[..=cutoff]);
    }
}

fn assert_metadata_reuse(
    join: &AnnularFrozenDomain,
    proof: &AnnularPrismConformance,
    expected: &[u8],
) {
    assert_eq!(
        certify_annular_prism_tolerances(join, proof, StoredToleranceCap::NoLimit, &mut Continue)
            .ok()
            .unwrap()
            .replay_bytes(),
        expected
    );
}
