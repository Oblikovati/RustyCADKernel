// SPDX-License-Identifier: GPL-2.0-only
use super::*;

fn reject_edit(edit: impl FnOnce(&mut PrismReceipt)) -> PrismMeasureFailure {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    edit(&mut receipt);
    measurement_attempt(&bridge, &shapes, &receipt, options(), || false)
        .err()
        .expect("edited binding must be incomplete")
}
#[test]
fn duplicate_parent_and_replaced_relation_rows_cannot_measure_old_body() {
    let error = reject_edit(|receipt| {
        let rows = &mut receipt.diagnostic.as_mut().unwrap().relations;
        rows[1] = rows[0].clone();
    });
    assert_hypothesis(error, "unique actual body parent relation");
}
#[test]
fn duplicate_native_parent_cannot_replace_composed_parent_multiset() {
    let error = reject_edit(|receipt| {
        let binding = &mut receipt.diagnostic.as_mut().unwrap().bindings[0];
        let parents = &mut binding.native.returned.generated;
        if let Some(first) = parents.first().cloned() {
            parents.push(first);
        } else {
            binding
                .native
                .returned
                .modified
                .push(binding.native.returned.modified[0].clone());
        }
    });
    assert_hypothesis(error, "all current composed parent dispositions");
}
#[test]
fn missing_duplicate_or_reparented_face_bindings_are_incomplete() {}
#[test]
fn reparented_context_is_not_current_body_membership() {
    let error = reject_edit(|receipt| {
        receipt.diagnostic.as_mut().unwrap().bindings.clear();
    });
    assert!(matches!(
        error,
        PrismMeasureFailure::Region(_) | PrismMeasureFailure::Chart(_)
    ));
    let error = reject_edit(|receipt| {
        receipt.diagnostic.as_mut().unwrap().bindings[0]
            .context
            .child_slots
            .push(999);
    });
    assert_hypothesis(error, "one actual image context");
}
#[test]
fn duplicate_query_role_context_is_not_new_evidence() {
    let error = reject_edit(|receipt| {
        let diagnostic = receipt.diagnostic.as_mut().unwrap();
        let old = &diagnostic.bindings[0];
        diagnostic.bindings.push(PrismImageBinding {
            native: old.native.clone(),
            context: old.context.clone(),
            transported: old.transported.clone(),
        });
    });
    assert_hypothesis(error, "one query-role-context history binding");
}
#[test]
fn fresh_body_cannot_borrow_preceding_body_history() {
    let (bridge, shapes) = fixture();
    let mut first = run(&bridge, &shapes);
    let second = run(&bridge, &shapes);
    first.diagnostic.as_mut().unwrap().body = second.diagnostic.unwrap().body;
    let error = measurement_attempt(&bridge, &shapes, &first, options(), || false)
        .err()
        .unwrap();
    assert_hypothesis(error, "one actual image context");
}
#[test]
fn mutable_status_rows_are_neither_required_nor_used_as_current_proof() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    receipt.region = None;
    receipt.diagnostic.as_mut().unwrap().surface_charts = None;
    let report = measured(&bridge, &shapes, &receipt);
    assert_eq!(report.charts.len(), 3);
    assert_eq!(report.region.selected_winding, 1);
    assert!(receipt.region.is_none());
    assert!(
        receipt
            .diagnostic
            .as_ref()
            .unwrap()
            .surface_charts
            .is_none()
    );
}
#[test]
fn changed_seed_cannot_reuse_old_unique_winding() {
    let error = reject_edit(|receipt| {
        receipt.limits.requested.seed = [99., 99.];
    });
    assert_hypothesis(error, "fresh selected component and exact windings");
}
#[test]
fn history_and_chart_order_never_define_measure_identity() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    let before = measured(&bridge, &shapes, &receipt);
    let diagnostic = receipt.diagnostic.as_mut().unwrap();
    diagnostic.bindings.reverse();
    diagnostic.relations.reverse();
    let after = measured(&bridge, &shapes, &receipt);
    assert_eq!(before.measures.first_cap, after.measures.first_cap);
    assert_eq!(before.measures.last_cap, after.measures.last_cap);
    assert_eq!(before.measures.signed_volume, after.measures.signed_volume);
    assert!(replay::same_charts(&before.charts, &after.charts).is_ok());
}

#[test]
fn missing_actual_face_cannot_reuse_complete_chart_rows() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    remove_actual_face(&receipt);
    let failure = measurement_attempt(&bridge, &shapes, &receipt, options(), || false)
        .err()
        .unwrap();
    assert_hypothesis(failure, "one actual image context");
}
fn remove_actual_face(receipt: &PrismReceipt) {
    let binding = receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|binding| {
            binding.context.occurrence.shape_type().unwrap()
                == rustycad_tkg3d::top_abs::ShapeEnum::Face
        })
        .unwrap();
    let mut parent = binding.context.ancestors.last().unwrap().clone();
    parent.set_free(true).unwrap();
    rustycad_tkbrep::brep::BRepBuilder::default()
        .remove(&mut parent, &binding.context.occurrence)
        .unwrap();
}
#[test]
fn extra_actual_face_cannot_borrow_previous_complete_coverage() {
    let (bridge, shapes) = fixture();
    let first = run(&bridge, &shapes);
    let second = run(&bridge, &shapes);
    add_foreign_face(&first, &second);
    let failure = measurement_attempt(&bridge, &shapes, &first, options(), || false)
        .err()
        .unwrap();
    assert_hypothesis(failure, "actual body relation coverage");
}
fn add_foreign_face(receipt: &PrismReceipt, foreign: &PrismReceipt) {
    let first = face_context(receipt);
    let mut parent = first.ancestors.last().unwrap().clone();
    parent.set_free(true).unwrap();
    rustycad_tkbrep::brep::BRepBuilder::default()
        .add(&mut parent, &face_context(foreign).occurrence)
        .unwrap();
}

fn face_context(receipt: &PrismReceipt) -> PrismBodyContext {
    receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .find(|binding| {
            binding.context.occurrence.shape_type().unwrap()
                == rustycad_tkg3d::top_abs::ShapeEnum::Face
        })
        .map(|binding| binding.context.clone())
        .unwrap()
}
