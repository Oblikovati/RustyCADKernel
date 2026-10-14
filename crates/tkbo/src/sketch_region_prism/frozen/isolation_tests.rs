// SPDX-License-Identifier: GPL-2.0-only
//! The same entry exercises failure/cancellation/reuse; immutable owned output has no source aliases.
use super::{tests::*, *};
use rustycad_tkernel::standard::Failure;

struct NamedFreezeStop {
    phase: FreezePhase,
    failure: bool,
    visited: bool,
}
impl FreezeProgress for NamedFreezeStop {
    fn checkpoint(&mut self, phase: FreezePhase) -> Result<bool, Failure> {
        if phase != self.phase {
            return Ok(false);
        }
        self.visited = true;
        if self.failure {
            return Err(Failure::new(
                "authored interruption; expected completed freeze",
            ));
        }
        Ok(true)
    }
}
#[test]
fn every_phase_cancellation_failure_and_reuse_leave_input_unchanged() {
    let (source, receipt) = authored_receipt();
    let bindings = authored_bindings(&receipt, &source);
    let before =
        captured(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress)).replay_bytes();
    for phase in [
        FreezePhase::Admission,
        FreezePhase::Identity,
        FreezePhase::Geometry,
        FreezePhase::Incidence,
        FreezePhase::Evidence,
        FreezePhase::Commit,
    ] {
        for failure in [false, true] {
            assert_stopped(&receipt, &bindings, phase, failure);
            let after = captured(freeze_prism(&receipt, &bindings, &mut NamedFreezeProgress));
            assert_eq!(before, after.replay_bytes());
        }
    }
}
fn assert_stopped(
    receipt: &super::super::PrismReceipt,
    bindings: &PrismPublicationBindings,
    phase: FreezePhase,
    failure: bool,
) {
    let mut progress = NamedFreezeStop {
        phase,
        failure,
        visited: false,
    };
    let outcome = freeze_prism(receipt, bindings, &mut progress);
    assert!(progress.visited);
    match outcome {
        PrismFreezeOutcome::Cancelled { phase: actual, .. } => assert!(!failure && actual == phase),
        PrismFreezeOutcome::Failed(error) => assert!(
            failure && error.phase == phase && matches!(error.cause, PrismFreezeCause::Native(_))
        ),
        _ => panic!("expected explicit cancellation/failure without output prefix"),
    }
}
#[test]
fn clone_mutation_is_isolated_from_published_values() {
    let original = fresh_projection();
    let bytes = original.replay_bytes();
    let mut copy = original.clone();
    copy.nodes[0].effective_tolerance = Some(123.);
    copy.uses.clear();
    copy.faces.clear();
    copy.history.derived.clear();
    assert_ne!(copy.replay_bytes(), bytes);
    assert_eq!(original.replay_bytes(), bytes);
}
#[test]
fn all_owned_output_types_are_send_and_sync_without_unsafe_impls() {
    fn assert_traits<T: Send + Sync>() {}
    assert_traits::<FrozenPrismProjection>();
    assert_traits::<PrismFreezeOutcome>();
    assert_traits::<PrismFreezeFailure>();
    assert_traits::<PrismFreezeCause>();
    assert_traits::<FrozenPrismNode>();
    assert_traits::<FrozenPrismUse>();
    assert_traits::<FrozenPrismFace>();
    assert_traits::<FrozenPrismTrim>();
    assert_traits::<FrozenPrismHistory>();
    assert_traits::<FrozenPrismLimits>();
    assert_traits::<super::graph::FrozenGraphIncidence>();
    assert_traits::<super::coverage::FrozenPrismCoverage>();
    assert_traits::<super::endpoints::FrozenEndpointProof>();
    assert_traits::<PrismPublicationBindings>();
    assert_traits::<NamedFreezeStop>();
}
