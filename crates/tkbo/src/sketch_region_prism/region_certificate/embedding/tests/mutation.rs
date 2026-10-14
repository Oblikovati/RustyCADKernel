// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::super::isolation_tests::{first_edge, pole};
use super::super::super::measurements::tests::{
    history_tests::{add_foreign_face, remove_actual_face},
    mutation_tests::{edit_target, live_face},
};
use super::*;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{edge, face},
};

#[test]
fn actual_source_copy_and_side_edits_refuse_and_cannot_change_prior_evidence() {
    for target in 0..3 {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        let before = embedded(&bridge, &shapes, &receipt);
        let regularity = before.regularity().clone();
        let rows = before.incidences().clone();
        edit_target(target, &shapes, &receipt);
        assert!(attempt(&bridge, &shapes, &receipt, options(), || false).is_err());
        assert_eq!(before.regularity(), &regularity);
        assert_eq!(before.incidences(), &rows);
    }
}
#[test]
fn actual_cap_trim_and_body_face_universe_edits_refuse() {
    for target in 0..3 {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        match target {
            0 => {
                let cap = live_face(&receipt, PrismImageRole::Last);
                BRepBuilder::default()
                    .range_on_face(&first_edge(&cap), &cap, 0., 0.5)
                    .unwrap();
            }
            1 => remove_actual_face(&receipt),
            _ => add_foreign_face(&receipt, &run(&bridge, &shapes)),
        }
        assert!(attempt(&bridge, &shapes, &receipt, options(), || false).is_err());
    }
}
#[test]
fn modified_history_role_parent_context_and_seed_refuse_old_success() {
    for target in 0..5 {
        let (bridge, shapes) = fixture();
        let mut receipt = run(&bridge, &shapes);
        edit_metadata(&mut receipt, target);
        let failure = attempt(&bridge, &shapes, &receipt, options(), || false)
            .err()
            .unwrap();
        assert_eq!(failure.phase, PrismEmbeddingPhase::CurrentBinding);
    }
}
fn edit_metadata(receipt: &mut PrismReceipt, target: usize) {
    if target == 4 {
        receipt.limits.requested.seed = [99., 99.];
        return;
    }
    let diagnostic = receipt.diagnostic.as_mut().unwrap();
    match target {
        0 | 1 => edit_face_metadata(diagnostic, target),
        2 => diagnostic.bindings[0].context.child_slots.push(999),
        _ => diagnostic.relations[1] = diagnostic.relations[0].clone(),
    }
}
fn edit_face_metadata(diagnostic: &mut PrismDiagnostic, target: usize) {
    let native = &mut diagnostic
        .bindings
        .iter_mut()
        .find(|row| {
            row.context.occurrence.shape_type().unwrap() == rustycad_tkg3d::top_abs::ShapeEnum::Face
        })
        .unwrap()
        .native;
    if target == 0 {
        native.role = match native.role {
            PrismImageRole::Last => PrismImageRole::First,
            _ => PrismImageRole::Last,
        };
        return;
    }
    native.returned.preserved.clear();
    native.returned.modified.clear();
    native.returned.generated.clear();
}
fn edit_embedding_operand(target: usize, shapes: &[Shape], receipt: &PrismReceipt) {
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
fn last_callback_mutation_is_caught_by_shared_final_rebind() {
    for target in 0..4 {
        let (bridge, shapes) = fixture();
        let receipt = run(&bridge, &shapes);
        let count = count_polls(&bridge, &shapes, &receipt);
        let mut poll = 0;
        let failure = attempt(&bridge, &shapes, &receipt, options(), || {
            poll += 1;
            if poll == count {
                edit_embedding_operand(target, &shapes, &receipt);
            }
            false
        })
        .err()
        .expect("late mutation must refuse");
        assert_eq!(poll, count);
        assert_eq!(failure.phase, PrismEmbeddingPhase::FinalBinding);
    }
}
fn count_polls(
    bridge: &PairedContactTransaction,
    shapes: &[Shape],
    receipt: &PrismReceipt,
) -> usize {
    let mut polls = 0;
    assert!(
        attempt(bridge, shapes, receipt, options(), || {
            polls += 1;
            false
        })
        .is_ok()
    );
    polls
}
#[test]
fn success_failure_cancel_leave_original_and_copy_unmodified() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let original = edge(&shapes[0]).unwrap();
    let copy = first_edge(&face(&receipt.copied.as_ref().unwrap().base).unwrap());
    let before = [pole(&original), pole(&copy)];
    embedded(&bridge, &shapes, &receipt);
    let mut policy = options();
    policy.regularity.max_stored_degree = 0;
    assert!(attempt(&bridge, &shapes, &receipt, policy, || false).is_err());
    assert!(attempt(&bridge, &shapes, &receipt, options(), || true).is_err());
    assert_eq!([pole(&original), pole(&copy)], before);
    embedded(&bridge, &shapes, &receipt);
}
