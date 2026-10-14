// SPDX-License-Identifier: GPL-2.0-only
//! ADR0039 starts from retained checker evidence; exact topology never rewrites that history.
use super::{
    alerts::FrozenNativeAlert::IntersectionFailedAtFailGravity,
    coverage::FrozenValidationCause::AnalyticNativeAlert, topology::*, *,
};
use crate::sketch_region_validation::StageCoverage;
use rustycad_tkernel::standard::Failure;

struct NamedBaselineTopologyProgress;
impl TopologyProgress for NamedBaselineTopologyProgress {
    fn checkpoint(&mut self, _: TopologyCheckpoint) -> Result<bool, Failure> {
        // No external activity participates in this owned historical baseline.
        Ok(false)
    }
}

#[test]
fn ordered_validation_baseline_keeps_historical_alert_after_exact_topology() {
    let projection = traversal_tests::ordered_projection();
    let before = projection.replay_bytes();
    assert!(matches!(
        certify_frozen_topology(&projection, &mut NamedBaselineTopologyProgress),
        FrozenTopologyOutcome::Certified(_)
    ));
    assert_eq!(projection.replay_bytes(), before);
    assert_eq!(
        projection.coverage().current,
        [StageCoverage::Incomplete; 4]
    );
    assert_historical_failure(&projection);
    assert!(!projection.is_valid_body());
}

fn assert_historical_failure(projection: &FrozenPrismProjection) {
    use StageCoverage::{Blocked, Incomplete, Passed};
    assert_eq!(
        projection.coverage().historical_unbound,
        [Passed, Passed, Incomplete, Blocked]
    );
    let alerts = projection
        .coverage()
        .findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.cause,
                AnalyticNativeAlert(IntersectionFailedAtFailGravity)
            )
        })
        .count();
    assert_eq!(alerts, 1);
}

#[test]
fn ordered_validation_baseline_does_not_supply_history_error_or_v1_traversal() {
    let projection = tests::fresh_projection();
    let before = projection.replay_bytes();
    assert!(projection.limits().achieved_error.is_none());
    assert!(!projection.history().retired_intermediate_unbound.is_empty());
    assert!(matches!(
        certify_frozen_topology(&projection, &mut NamedBaselineTopologyProgress),
        FrozenTopologyOutcome::Incomplete(_)
    ));
    assert_eq!(projection.replay_bytes(), before);
    assert!(!projection.is_valid_body());
}
