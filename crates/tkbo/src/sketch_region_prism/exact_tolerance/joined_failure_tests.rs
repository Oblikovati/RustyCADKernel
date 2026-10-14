// SPDX-License-Identifier: GPL-2.0-only
use super::{closure_scalar::*, joined_progress::ConformanceWork, joined_test_support::*, *};
use crate::sketch_region_prism::frozen::proof_join::JoinedClosurePointOperand;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;
fn classified_closure(
    join: &JoinedPrismProof,
    raw: &JoinedClosurePointOperand,
    cause: ExactClosureFailure,
) -> JoinedConformanceFailure {
    let mut work = ConformanceWork::new();
    work.phase(JoinedConformancePhase::Closures, &mut NeverCancel)
        .ok()
        .unwrap();
    work.failed(
        JoinedConformanceCause::Closure {
            contact: raw.contact,
            original: raw.original.clone(),
            stage: raw.stage,
            cause: Box::new(cause),
        },
        join,
    )
}
#[test]
fn exact_closure_discrepancy_is_failed_with_original_semantic_context() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let raw = &join.raw_closure_points()[0];
    let mut displaced = raw.local_point;
    displaced[0] += 0.125;
    let cause = raw_failure(&join, displaced, raw.placement);
    let failure = classified_closure(&join, raw, cause);
    assert_eq!(failure.classification(), JoinedEvidenceClass::Failed);
    assert_eq!(failure.phase, JoinedConformancePhase::Closures);
    assert_eq!(failure.binding, join.replay_bytes());
    assert!(
        matches!(failure.cause, JoinedConformanceCause::Closure {contact,ref original,stage,..}
        if contact==raw.contact && original==&raw.original && stage==raw.stage)
    );
}
#[test]
fn unusable_frame_is_incomplete_without_converting_it_to_a_metric_witness() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let raw = &join.raw_closure_points()[0];
    let mut singular = raw.placement;
    singular.scale = 0.;
    let cause = raw_failure(&join, raw.local_point, singular);
    let failure = classified_closure(&join, raw, cause);
    assert_eq!(failure.classification(), JoinedEvidenceClass::Incomplete);
    assert!(failure.partial.closures.is_empty());
}
#[test]
fn source_policy_error_retains_original_root_options_and_incomplete_class() {
    let join = JoinFixture::new().compose(|| false).ok().unwrap();
    let raw = &join.raw_closure_points()[0];
    let cause = policy_failure(&join, raw);
    let failure = classified_closure(&join, raw, cause);
    assert_eq!(failure.classification(), JoinedEvidenceClass::Incomplete);
    assert!(
        matches!(failure.cause, JoinedConformanceCause::Closure {cause,..}
        if matches!(*cause, ExactClosureFailure::Endpoint(_)))
    );
}

fn raw_failure(
    join: &JoinedPrismProof,
    point: [f64; 3],
    placement: MetricEmbedding<3>,
) -> ExactClosureFailure {
    let query = join.closure_query();
    certify_exact_source_point(query.source_query(), query.bounds.clone(), point, placement)
        .err()
        .unwrap()
}
fn policy_failure(join: &JoinedPrismProof, raw: &JoinedClosurePointOperand) -> ExactClosureFailure {
    let mut query = join.closure_query().source_query();
    query.residual.options.version = 999;
    certify_exact_source_point(
        query,
        join.closure_query().bounds.clone(),
        raw.local_point,
        raw.placement,
    )
    .err()
    .unwrap()
}
