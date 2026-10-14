// SPDX-License-Identifier: GPL-2.0-only
//! Existing copier observations are retained without changing flattened legacy relations.
use super::{tests::*, *};
use crate::sketch_region::ArrangementRelation;

#[test]
fn actual_selected_copy_pairs_preserve_every_original_relation_class() {
    let receipt = cubic_receipt([1., 1.], &mut progress(None, None));
    let selected = receipt.input.selected.as_ref().unwrap();
    let copied = receipt.copied.as_ref().unwrap();
    assert_eq!(copied.observations.len(), selected.relations.len());
    for observation in &copied.observations {
        assert_observed_pair(&selected.relations, copied, observation);
    }
}
fn assert_observed_pair(
    selected: &[ArrangementRelation],
    copied: &PrismCopiedSelection,
    observation: &PrismCopyObservation,
) {
    let selected_row = selected
        .iter()
        .find(|row| row.output.is_equal(&observation.selected.output))
        .unwrap();
    assert_relation(selected_row, &observation.selected);
    assert!(
        !observation
            .selected
            .output
            .is_partner(&observation.native_image)
    );
    assert!(observation.native_image.is_same(&observation.copied_image));
    assert_eq!(
        observation.copied_image.orientation(),
        selected_row.output.orientation()
    );
    assert_legacy_relation(copied, observation);
}
fn assert_legacy_relation(copied: &PrismCopiedSelection, observation: &PrismCopyObservation) {
    let expected = crate::sketch_region_candidates::composed_copy_relation(
        &observation.selected,
        &observation.copied_image,
    );
    let actual = copied
        .relations
        .iter()
        .find(|row| row.output.is_equal(&observation.copied_image))
        .unwrap();
    assert_relation(actual, &expected);
}
fn assert_relation(first: &ArrangementRelation, second: &ArrangementRelation) {
    assert!(first.output.is_equal(&second.output));
    assert_eq!(first.preserved, second.preserved);
    assert_eq!(first.modified, second.modified);
    assert_eq!(first.generated, second.generated);
}
#[test]
fn generated_selected_parents_remain_generated_in_observation_and_legacy_copy() {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let mut batch = crate::sketch_region_candidates::tests::candidates(&[source]);
    for candidate in &mut batch.candidates {
        for relation in &mut candidate.relations {
            relation.generated.append(&mut relation.modified);
        }
    }
    let receipt = retain_selected_prism(batch, request([1., 1.]), &mut progress(None, None));
    let copied = receipt.copied.as_ref().unwrap();
    for observation in &copied.observations {
        assert!(!observation.selected.generated.is_empty());
        assert!(observation.selected.modified.is_empty());
        assert_legacy_relation(copied, observation);
    }
}
#[test]
fn stopping_after_copy_retains_observations_without_creating_a_body() {
    let receipt = cubic_receipt([1., 1.], &mut progress(Some(PrismPhase::Sweep), None));
    assert!(matches!(
        receipt.completion,
        PrismCompletion::Cancelled(PrismPhase::Sweep)
    ));
    assert!(!receipt.copied.unwrap().observations.is_empty());
    assert!(receipt.diagnostic.is_none());
}
#[test]
fn separate_operations_never_reuse_selected_to_copy_observations() {
    let first = cubic_receipt([1., 1.], &mut progress(None, None));
    let second = cubic_receipt([1., 1.], &mut progress(None, None));
    for first in &first.copied.unwrap().observations {
        for second in &second.copied.as_ref().unwrap().observations {
            assert!(!first.selected.output.is_partner(&second.selected.output));
            assert!(!first.copied_image.is_partner(&second.copied_image));
        }
    }
}
