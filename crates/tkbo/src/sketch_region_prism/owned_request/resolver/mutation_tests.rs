// SPDX-License-Identifier: GPL-2.0-only
use super::{invocation_tests::PublicationObserver, source_tests::*, *};
use crate::sketch_region_prism::publication::{mutation_tests::*, *};
use rustycad_tkbrep::topods::Shape;

#[test]
fn final_original_copied_and_output_point_mutations_refuse_without_stale_map_issuance() {
    for target in 0..3 {
        let prepared = operation(false);
        let invocation = resolved(&prepared);
        let accepted = published(&invocation);
        let names = body_map::issue_resolved_prism_names(&accepted).unwrap();
        let before = names.replay_bytes().to_vec();
        let mut edit = vertex_edit(mutation_target(&prepared, target));
        let failure = invocation.publish(&mut edit).err().unwrap();
        assert_eq!(failure.failure().phase, PrismPublicationPhase::Commit);
        assert_eq!(edit.commits, 1);
        assert_eq!(names.replay_bytes(), before);
        assert!(names.binds(accepted.result()));
        assert_eq!(accepted.request().sources[0].world_poles[0], [0.; 3]);
        assert!(resolve_invocation(&prepared, plan_for(&prepared), &mut || false).is_err());
    }
}
fn mutation_target(prepared: &PreparedPolynomialPrism, target: u8) -> Shape {
    let receipt = prepared.retained.prism.as_ref().unwrap();
    let root = match target {
        0 => &prepared.retained.source.as_ref().unwrap().edge,
        1 => &receipt.copied.as_ref().unwrap().base,
        _ => &receipt.diagnostic.as_ref().unwrap().body,
    };
    first_vertex(root)
}
#[test]
fn every_actual_publication_checkpoint_cancel_or_failure_preserves_request_and_allows_reuse() {
    let prepared = operation(false);
    let invocation = resolved(&prepared);
    let mut baseline = PublicationObserver::default();
    let accepted = invocation.publish(&mut baseline).ok().unwrap();
    for stop in 0..baseline.seen.len() {
        for failed in [false, true] {
            assert_terminal_prefix(&invocation, &baseline.seen, stop, failed);
        }
    }
    let repeated = published(&invocation);
    assert_eq!(
        accepted.result().replay_bytes(),
        repeated.result().replay_bytes()
    );
    println!(
        "ACTUAL_RESOLVER_PUBLICATION_CHECKPOINTS={}",
        baseline.seen.len()
    );
}
fn assert_terminal_prefix(
    invocation: &ResolvedPrismInvocation<'_>,
    baseline: &[PrismPublicationCheckpoint],
    stop: usize,
    failed: bool,
) {
    let mut observer = PublicationObserver {
        stop: (!failed).then_some(stop),
        fail: failed.then_some(stop),
        ..Default::default()
    };
    let packet = invocation.publish(&mut observer).err().unwrap();
    assert_eq!(observer.seen, baseline[..=stop]);
    assert_eq!(
        packet.request().sources[0].world_poles,
        super::super::test_fixture::request().sources[0].world_poles
    );
    assert_terminal_class(packet.failure().class, failed);
}

fn resolved(prepared: &PreparedPolynomialPrism) -> ResolvedPrismInvocation<'_> {
    checked(resolve_invocation(
        prepared,
        plan_for(prepared),
        &mut || false,
    ))
}
fn published(invocation: &ResolvedPrismInvocation<'_>) -> ResolvedPrismPublication {
    invocation
        .publish(&mut PublicationObserver::default())
        .ok()
        .unwrap()
}
fn assert_terminal_class(actual: PrismPublicationClass, failed: bool) {
    let expected = if failed {
        PrismPublicationClass::Failed
    } else {
        PrismPublicationClass::Cancelled
    };
    assert_eq!(actual, expected);
}
