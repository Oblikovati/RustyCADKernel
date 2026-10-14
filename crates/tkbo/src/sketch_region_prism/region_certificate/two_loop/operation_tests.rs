// SPDX-License-Identifier: GPL-2.0-only
use super::{fixture::*, fixture_bindings, operation::*, outcome::*, request::*};
use rustycad_tkernel::standard::Failure;
pub(super) struct OperationProgress;
impl TwoLoopRegionProgress for OperationProgress {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, Failure> {
        Ok(false)
    }
}
pub(super) fn inspect<'a>(
    fixture: &'a TwoLoopFixture,
    progress: &mut dyn TwoLoopRegionProgress,
) -> TwoLoopRegionOutcome<'a> {
    let sources = fixture.sources();
    let identities = fixture_bindings::identities(fixture);
    inspect_two_loop_region(
        &TwoLoopRegionRequest {
            bridge: &fixture.bridge,
            sources: &sources,
            identities: &identities,
            inner_interior: [1., 1.],
            material_seed: [0.25, 0.25],
            requested: fixture_bindings::options(),
        },
        progress,
    )
}
pub(super) fn accepted(outcome: TwoLoopRegionOutcome<'_>) -> Box<TwoLoopRegionCertificate> {
    match outcome {
        TwoLoopRegionOutcome::Inspected(proof) => proof,
        TwoLoopRegionOutcome::Incomplete(failure)
        | TwoLoopRegionOutcome::Failed(failure)
        | TwoLoopRegionOutcome::Cancelled(failure) => {
            panic!("operation {:?}: {:?}", failure.stage, failure.cause)
        }
    }
}
#[test]
fn complete_two_loop_inspection_owns_three_domains_and_every_candidate_member() {
    let fixture = TwoLoopFixture::nested();
    let proof = accepted(inspect(&fixture, &mut OperationProgress));
    assert_eq!(proof.domains.len(), 3);
    assert_eq!(proof.candidates.len(), 3);
    assert_eq!(
        proof
            .candidates
            .iter()
            .map(|c| c.members.len())
            .sum::<usize>(),
        19
    );
    assert_eq!(proof.nesting.native_signs, [1, 1]);
    assert_eq!(proof.sources.len(), 2);
    assert!(!proof.replay.is_empty());
}
#[test]
fn complete_region_domain_owned_result_is_send_sync() {
    fn assert_immutable<T: Send + Sync>() {}
    assert_immutable::<TwoLoopRegionCertificate>();
}

#[test]
fn borrowed_domain_views_expose_all_operands_and_explicit_remaining_gates() {
    let fixture = TwoLoopFixture::nested();
    let proof = accepted(inspect(&fixture, &mut OperationProgress));
    assert_eq!(proof.operation_id(), 77);
    assert_eq!(proof.selected_candidate_id(), 301);
    assert_eq!(proof.sources().len(), 2);
    assert_eq!(proof.domains().len(), 3);
    assert_eq!(proof.candidates().len(), 3);
    assert_eq!(proof.nesting().native_signs, [1, 1]);
    assert!(
        proof
            .census_settings()
            .chart_rank
            .compare_f64(0.)
            .unwrap()
            .is_gt()
    );
    assert_eq!(proof.requested(), fixture_bindings::options());
    assert!(proof.retained_units() > 0);
    assert!(!proof.replay_bytes().is_empty());
    assert!(
        proof
            .limitations()
            .contains(&super::TwoLoopRegionLimitation::ComposedCopyAndDeletionHistory)
    );
    assert!(
        proof
            .limitations()
            .contains(&super::TwoLoopRegionLimitation::OrderedBodyValidation)
    );
}
