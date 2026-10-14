// SPDX-License-Identifier: GPL-2.0-only
use super::{joined_test_support::*, *};
#[derive(Default)]
struct StopConformance {
    stop: usize,
    phases: Vec<JoinedConformancePhase>,
}
impl JoinedConformanceProgress for StopConformance {
    fn cancelled(&mut self, phase: JoinedConformancePhase) -> bool {
        self.phases.push(phase);
        self.phases.len() == self.stop
    }
}
#[derive(Default)]
struct StopTolerance {
    stop: usize,
    phases: Vec<JoinedTolerancePhase>,
}
impl JoinedToleranceProgress for StopTolerance {
    fn cancelled(&mut self, phase: JoinedTolerancePhase) -> bool {
        self.phases.push(phase);
        self.phases.len() == self.stop
    }
}
#[test]
fn every_observed_conformance_callback_can_cancel_without_completed_geometry() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut baseline = StopConformance::default();
    assert!(certify_joined_prism_conformance(&join, &mut baseline).is_ok());
    for stop in 1..=baseline.phases.len() {
        let mut progress = StopConformance {
            stop,
            ..Default::default()
        };
        let refusal = certify_joined_prism_conformance(&join, &mut progress)
            .err()
            .unwrap();
        assert_eq!(refusal.classification(), JoinedEvidenceClass::Cancelled);
        assert_eq!(refusal.phase, baseline.phases[stop - 1]);
        assert_eq!(progress.phases, baseline.phases[..stop]);
        assert_eq!(refusal.binding, join.replay_bytes());
    }
    assert!(conformance(&join).binds(&join));
}
#[test]
fn every_metadata_callback_cancels_with_exact_completed_prefix() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let proof = conformance(&join);
    let mut baseline = StopTolerance::default();
    assert!(
        certify_joined_prism_tolerances(&join, &proof, StoredToleranceCap::NoLimit, &mut baseline)
            .is_ok()
    );
    for stop in 1..=baseline.phases.len() {
        assert_metadata_stop(&join, &proof, &baseline, stop);
    }
}

fn assert_metadata_stop(
    join: &JoinedPrismProof,
    proof: &JoinedPrismConformance,
    baseline: &StopTolerance,
    stop: usize,
) {
    let mut progress = StopTolerance {
        stop,
        ..Default::default()
    };
    let refusal =
        certify_joined_prism_tolerances(join, proof, StoredToleranceCap::NoLimit, &mut progress)
            .err()
            .unwrap();
    assert_eq!(refusal.classification(), JoinedEvidenceClass::Cancelled);
    assert_eq!(progress.phases, baseline.phases[..stop]);
    assert_eq!(refusal.metadata.len(), stop.saturating_sub(2).min(8));
}

#[test]
fn exact_conformance_callback_trace_baseline() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let mut geometry = StopConformance::default();
    let proof = certify_joined_prism_conformance(&join, &mut geometry)
        .ok()
        .unwrap();
    let mut metadata = StopTolerance::default();
    assert!(
        certify_joined_prism_tolerances(&join, &proof, StoredToleranceCap::NoLimit, &mut metadata)
            .is_ok()
    );
    println!(
        "exact-conformance-callback-baseline:{:?}/{:?}",
        geometry.phases, metadata.phases
    );
}
