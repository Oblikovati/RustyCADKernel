// SPDX-License-Identifier: GPL-2.0-only
//! Structural query receipts preserve empty answers; no history coverage is inferred here.
use super::{tests::*, *};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;

fn queries(receipt: &PrismReceipt) -> &[PrismSweepQueryObservation] {
    let PrismSweepQueries::Captured(rows) = &receipt.diagnostic.as_ref().unwrap().sweep_queries
    else {
        panic!("query stage absent; expected actual captured sweep queries");
    };
    rows
}
#[test]
fn every_copied_member_retains_actual_empty_images_and_false_retirement() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let rows = queries(&receipt);
    assert_eq!(rows.len(), 5);
    assert_eq!(rows.iter().filter(|row| row.retired).count(), 1);
    assert_eq!(rows.iter().filter(|row| !row.retired).count(), 4);
    assert!(rows.iter().any(|row| row.generated.is_empty()));
    for row in rows {
        assert_eq!(
            row.retired,
            row.query.output.shape_type().unwrap() == ShapeEnum::Wire
        );
        assert_exact_query_parent_row(&receipt, row);
    }
}
fn assert_exact_query_parent_row(receipt: &PrismReceipt, row: &PrismSweepQueryObservation) {
    let matches: Vec<_> = receipt
        .copied
        .as_ref()
        .unwrap()
        .relations
        .iter()
        .filter(|source| source.output.is_equal(&row.query.output))
        .collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].preserved, row.query.preserved);
    assert_eq!(matches[0].modified, row.query.modified);
    assert_eq!(matches[0].generated, row.query.generated);
}
#[test]
fn exhaustive_query_images_and_existing_native_bindings_match_both_ways() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let diagnostic = receipt.diagnostic.as_ref().unwrap();
    for binding in &diagnostic.bindings {
        let rows: Vec<_> = queries(&receipt)
            .iter()
            .filter(|row| row.query.output.is_equal(&binding.native.query))
            .collect();
        assert_eq!(rows.len(), 1);
        assert!(
            images(rows[0], binding.native.role)
                .iter()
                .any(|image| image.is_equal(&binding.native.returned.output))
        );
    }
    for row in queries(&receipt) {
        assert_images_bound(row, &diagnostic.bindings);
    }
}
fn images(row: &PrismSweepQueryObservation, role: PrismImageRole) -> Vec<&Shape> {
    match role {
        PrismImageRole::Generated => row.generated.iter().collect(),
        PrismImageRole::First => [&row.first].into_iter().filter(|s| !s.is_null()).collect(),
        PrismImageRole::Last => [&row.last].into_iter().filter(|s| !s.is_null()).collect(),
    }
}
fn assert_images_bound(row: &PrismSweepQueryObservation, bindings: &[PrismImageBinding]) {
    for role in [
        PrismImageRole::Generated,
        PrismImageRole::First,
        PrismImageRole::Last,
    ] {
        for image in images(row, role) {
            assert!(bindings.iter().any(|binding| binding.native.role == role
                && binding.native.query.is_equal(&row.query.output)
                && binding.native.returned.output.is_equal(image)));
        }
    }
}
#[test]
fn history_cancellation_and_failure_leave_query_stage_unknown() {
    for (stop, fail) in [
        (Some(PrismPhase::History), None),
        (None, Some(PrismPhase::History)),
    ] {
        let receipt = cubic_receipt([1., 1.], &mut progress(stop, fail));
        assert!(matches!(
            receipt.diagnostic.as_ref().unwrap().sweep_queries,
            PrismSweepQueries::NotReached
        ));
    }
}
#[test]
fn captured_zero_queries_remain_distinct_from_unknown_and_do_not_complete_history() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let mut batch = crate::sketch_region_candidates::tests::candidates(&[source]);
    for candidate in &mut batch.candidates {
        candidate.relations.clear();
    }
    let receipt = retain_selected_prism(batch, request([1., 1.]), &mut progress(None, None));
    assert!(queries(&receipt).is_empty());
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    assert!(
        receipt
            .gaps
            .iter()
            .any(|gap| matches!(gap, PrismGap::UnattributedSweepOccurrence(_)))
    );
}

#[test]
fn native_wire_retirement_keeps_observed_caps_as_separate_continuations() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let wire = queries(&receipt).iter().find(|row| row.retired).unwrap();
    // make_prism::is_deleted reports generated-shape usage, independently of retained caps.
    assert!(wire.generated.is_empty());
    assert!(!wire.first.is_null());
    assert!(!wire.last.is_null());
}
#[test]
fn explicit_null_cap_and_empty_generated_answers_have_no_positive_images() {
    let row = PrismSweepQueryObservation {
        query: crate::sketch_region::ArrangementRelation {
            output: Shape::default(),
            preserved: vec![],
            modified: vec![],
            generated: vec![],
        },
        generated: vec![],
        first: Shape::default(),
        last: Shape::default(),
        retired: false,
    };
    for role in [
        PrismImageRole::Generated,
        PrismImageRole::First,
        PrismImageRole::Last,
    ] {
        assert!(images(&row, role).is_empty());
    }
    assert!(!row.retired);
}
