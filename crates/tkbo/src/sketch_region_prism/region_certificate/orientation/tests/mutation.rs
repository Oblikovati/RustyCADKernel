// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::measurements::tests::mutation_tests::{edit_target, live_face};
use super::*;
use crate::sketch_region_prism::isolation_tests::{first_edge, pole};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{edge, face},
};

#[test]
fn source_copy_and_actual_chart_changes_never_reuse_old_orientation() {
    for target in 0..3 {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        let before = oriented(&bridge, &shapes, &receipt);
        let bytes = replay_tests::encoded(&before);
        edit_target(target, &shapes, &receipt);
        let error = attempt(&bridge, &shapes, &receipt, options(), || false)
            .err()
            .unwrap();
        assert_eq!(error.phase, PrismOrientationPhase::Embedding);
        assert_eq!(replay_tests::encoded(&before), bytes);
    }
}
pub(super) fn count_polls(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
) -> usize {
    let mut count = 0;
    assert!(
        attempt(bridge, shapes, receipt, options(), || {
            count += 1;
            false
        })
        .is_ok()
    );
    count
}
#[test]
fn final_orientation_callback_mutation_is_rejected_after_embedding_success() {
    for target in 0..4 {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        let cutoff = count_polls(&bridge, &shapes, &receipt);
        let mut count = 0;
        let error = attempt(&bridge, &shapes, &receipt, options(), || {
            count += 1;
            if count == cutoff {
                mutate_operand(target, &shapes, &receipt);
            }
            false
        })
        .err()
        .unwrap();
        assert_eq!(count, cutoff);
        assert_eq!(error.phase, PrismOrientationPhase::FinalBinding);
        assert!(error.original.is_some());
    }
}
fn mutate_operand(target: usize, shapes: &[Shape], receipt: &PrismReceipt) {
    if target < 3 {
        edit_target(target, shapes, receipt);
        return;
    }
    let cap = live_face(receipt, PrismImageRole::Last);
    BRepBuilder::default()
        .range_on_face(&first_edge(&cap), &cap, 0., 0.5)
        .unwrap();
}
#[test]
fn cancellation_initial_embedding_final_and_orientation_final_do_not_poison_retry() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let count = count_polls(&bridge, &shapes, &receipt);
    let before = replay_tests::encoded(&oriented(&bridge, &shapes, &receipt));
    for cutoff in [1, count - 1, count] {
        let mut polls = 0;
        assert!(
            attempt(&bridge, &shapes, &receipt, options(), || {
                polls += 1;
                polls == cutoff
            })
            .is_err()
        );
        assert_eq!(
            replay_tests::encoded(&oriented(&bridge, &shapes, &receipt)),
            before
        );
    }
}
#[test]
fn inspection_leaves_original_copy_and_candidate_cache_unchanged() {
    use crate::sketch_region_prism::isolation_tests::{assert_original_mesh, attached_mesh};
    let (bridge, shapes) = fixture();
    let candidate = &borrowed_batch(&bridge).unwrap().candidates[0];
    let mesh = attached_mesh(&candidate.face);
    let receipt = run(&bridge, &shapes);
    let original = edge(&shapes[0]).unwrap();
    let copied = first_edge(&face(&receipt.copied.as_ref().unwrap().base).unwrap());
    let before = [pole(&original), pole(&copied)];
    for mode in 0..3 {
        exercise_mode(&bridge, &shapes, &receipt, mode);
        assert_eq!([pole(&original), pole(&copied)], before);
        assert_original_mesh(&mesh);
    }
}
fn exercise_mode(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
    mode: usize,
) {
    let mut requested = options();
    if mode == 1 {
        requested.embedding.regularity.max_stored_degree = 0;
    }
    let result = attempt(bridge, shapes, receipt, requested, || mode == 2);
    assert_eq!(result.is_ok(), mode == 0);
}
#[test]
fn changed_body_root_orientation_cannot_borrow_old_cumulative_keys() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    receipt.diagnostic.as_mut().unwrap().body.reverse();
    let error = attempt(&bridge, &shapes, &receipt, options(), || false)
        .err()
        .unwrap();
    assert_eq!(error.phase, PrismOrientationPhase::Embedding);
}
