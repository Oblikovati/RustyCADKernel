// SPDX-License-Identifier: GPL-2.0-only
use super::super::{traversal_tests::ordered_projection as fresh_projection, *};
use super::*;
use rustycad_tkernel::standard::Failure;
#[derive(Default)]
pub(super) struct NamedTopologyProgress {
    pub(super) seen: Vec<TopologyCheckpoint>,
    pub(super) cancel: Option<TopologyCheckpoint>,
    pub(super) fail: Option<TopologyCheckpoint>,
}
impl TopologyProgress for NamedTopologyProgress {
    fn checkpoint(&mut self, point: TopologyCheckpoint) -> Result<bool, Failure> {
        self.seen.push(point);
        if self.fail == Some(point) {
            return Err(Failure::new(
                "named topology callback failure; expected successful callback",
            ));
        }
        Ok(self.cancel == Some(point))
    }
}
pub(super) fn certified(source: &FrozenPrismProjection) -> Box<FrozenTopologyCertificate> {
    match certify_frozen_topology(source, &mut NamedTopologyProgress::default()) {
        FrozenTopologyOutcome::Certified(report) => report,
        failure => panic!("{failure:?}; expected complete combinatorial certificate"),
    }
}
pub(super) fn rejected(
    source: &FrozenPrismProjection,
    expected: TopologyInvariant,
) -> TopologyFailure {
    let before = source.replay_bytes();
    let FrozenTopologyOutcome::Failed(failure) =
        certify_frozen_topology(source, &mut NamedTopologyProgress::default())
    else {
        panic!("expected witnessed combinatorial failure at {expected:?}");
    };
    assert_eq!(source.replay_bytes(), before);
    assert_eq!(failure.checkpoint.invariant, expected);
    failure
}
#[test]
fn actual_owned_prism_has_complete_discrete_evidence_without_body_promotion() {
    let source = fresh_projection();
    let before = source.replay_bytes();
    let report = certified(&source);
    assert_eq!(report.version(), 1);
    assert_eq!(report.counts, [2, 3, 6, 3, 3, 1, 1]);
    assert_eq!(report.invariants().len(), 6);
    assert_eq!(
        report.achieved_error(),
        TopologyNumericalError::NotApplicable
    );
    assert_eq!(
        report.evidence_kind(),
        TopologyEvidenceKind::ExactCombinatorial
    );
    assert!(report.binds(&source));
    assert_eq!(source.replay_bytes(), before);
    unchanged_body_coverage(&source);
}
fn unchanged_body_coverage(source: &FrozenPrismProjection) {
    assert!(!source.is_valid_body());
    assert_eq!(
        source.coverage.current,
        [crate::sketch_region_validation::StageCoverage::Incomplete; 4]
    );
}

#[test]
fn every_observed_checkpoint_can_cancel_or_fail_without_partial_certificate() {
    let source = fresh_projection();
    let before = source.replay_bytes();
    let mut progress = NamedTopologyProgress::default();
    assert!(matches!(
        certify_frozen_topology(&source, &mut progress),
        FrozenTopologyOutcome::Certified(_)
    ));
    assert_eq!(
        progress.seen.last().unwrap().invariant,
        TopologyInvariant::Commit
    );
    for point in progress.seen {
        check_cancel_and_failure(&source, point);
        assert_eq!(source.replay_bytes(), before);
    }
}
fn check_cancel_and_failure(source: &FrozenPrismProjection, point: TopologyCheckpoint) {
    let mut cancel = NamedTopologyProgress {
        cancel: Some(point),
        ..Default::default()
    };
    assert!(matches!(certify_frozen_topology(source, &mut cancel),
        FrozenTopologyOutcome::Cancelled { operation, checkpoint } if operation == source.operation && checkpoint == point));
    check_callback_failure(source, point);
}
fn check_callback_failure(source: &FrozenPrismProjection, point: TopologyCheckpoint) {
    let mut fail = NamedTopologyProgress {
        fail: Some(point),
        ..Default::default()
    };
    let FrozenTopologyOutcome::Incomplete(failure) = certify_frozen_topology(source, &mut fail)
    else {
        panic!("callback failure must remain incomplete");
    };
    assert_eq!(failure.checkpoint, point);
    assert!(matches!(
        failure.witness.actual,
        TopologyActual::Callback(_)
    ));
}

#[test]
fn source_binding_observes_owned_mutations_while_report_remains_unchanged() {
    let mut source = fresh_projection();
    let report = certified(&source);
    let bytes = report.replay_bytes();
    source.nodes[0].raw_tolerance = Some(9.);
    assert!(!report.binds(&source));
    assert_eq!(report.replay_bytes(), bytes);
}
#[test]
fn unsupported_version_orientation_and_bounded_work_remain_incomplete() {
    for variant in 0..3 {
        let mut source = fresh_projection();
        match variant {
            0 => source.version = 1,
            1 => source.uses[0].orientation = rustycad_tkg3d::top_abs::Orientation::Internal,
            _ => source.uses = vec![source.uses[0].clone(); 4097],
        }
        let before = source.replay_bytes();
        assert!(matches!(
            certify_frozen_topology(&source, &mut NamedTopologyProgress::default()),
            FrozenTopologyOutcome::Incomplete(_)
        ));
        assert_eq!(source.replay_bytes(), before);
    }
}
