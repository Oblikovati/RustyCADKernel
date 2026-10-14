// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    PrismReceipt,
    contexts::body_contexts,
    region_certificate::{current_binding::*, tests},
};
use super::{census::*, census_types::*};
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn fixture() -> (PrismReceipt, CurrentPrismBinding) {
    let (bridge, shapes) = tests::fixture();
    let receipt = tests::run(&bridge, &shapes);
    let binding =
        capture_current_prism_binding(&bridge, &binding_sources(&shapes), &receipt, &mut || false)
            .ok()
            .expect("current authored source/geometry binding");
    (receipt, binding)
}

#[test]
fn census_retains_all_directed_uses_and_shares_only_physical_metadata_entities() {
    let (receipt, binding) = fixture();
    let census = capture_census(&receipt, &binding)
        .ok()
        .expect("complete census");
    assert_eq!(census.contexts.len(), 26);
    assert_eq!(census.entities.len(), 8);
    assert_eq!(census.trims.len(), 6);
    assert_eq!(census.history.len(), 26);
    assert_metadata_counts(&census);
    assert!(census.trims.iter().all(|row| !row.parents.is_empty()));
}

fn assert_metadata_counts(census: &PrismToleranceCensus) {
    for (kind, physical, directed) in [
        (ShapeEnum::Face, 3, 3),
        (ShapeEnum::Edge, 3, 6),
        (ShapeEnum::Vertex, 2, 12),
    ] {
        let rows: Vec<_> = census
            .entities
            .iter()
            .filter(|row| row.kind == kind)
            .collect();
        assert_eq!(rows.len(), physical);
        assert_eq!(
            rows.iter().map(|row| row.contexts.len()).sum::<usize>(),
            directed
        );
    }
}

#[test]
fn missing_trim_proof_cannot_be_hidden_by_complete_face_counts() {
    let (receipt, mut binding) = fixture();
    binding.charts[0].captured.trims.pop().unwrap();
    let error = capture_census(&receipt, &binding)
        .err()
        .expect("missing use rejected");
    assert_count(error, "bound chart trim for directed edge", 0, 1, true);
}

#[test]
fn missing_body_edge_use_cannot_be_hidden_by_retained_trim_proof() {
    let (receipt, binding) = fixture();
    let body = &receipt.diagnostic.as_ref().unwrap().body;
    let mut contexts = body_contexts(body).ok().unwrap();
    let index = contexts
        .iter()
        .position(|row| row.occurrence.shape_type().unwrap() == ShapeEnum::Edge)
        .unwrap();
    contexts.remove(index);
    let error = capture_census_from_contexts(&contexts, &binding, relations(&receipt))
        .err()
        .expect("missing body use rejected");
    assert_count(error, "body occurrence for captured trim", 0, 1, false);
}

#[test]
fn duplicate_context_is_rejected_instead_of_collapsed_into_a_physical_entity() {
    let (receipt, binding) = fixture();
    let mut contexts = body_contexts(&receipt.diagnostic.as_ref().unwrap().body)
        .ok()
        .unwrap();
    contexts.push(contexts.last().unwrap().clone());
    let error = capture_census_from_contexts(&contexts, &binding, relations(&receipt))
        .err()
        .expect("duplicate use rejected");
    assert_count(error, "unique directed body context", 2, 1, true);
}

#[test]
fn missing_face_chart_keeps_actual_use_attribution() {
    let (receipt, mut binding) = fixture();
    binding.charts.remove(0);
    let error = capture_census(&receipt, &binding)
        .err()
        .expect("missing chart rejected");
    assert_count(error, "current chart for body face", 0, 1, true);
}

#[test]
fn census_names_come_from_actual_occurrences_after_storage_order_changes() {
    let (receipt, binding) = fixture();
    let mut contexts = body_contexts(&receipt.diagnostic.as_ref().unwrap().body)
        .ok()
        .unwrap();
    let first = capture_census_from_contexts(&contexts, &binding, relations(&receipt))
        .ok()
        .unwrap();
    contexts.reverse();
    let second = capture_census_from_contexts(&contexts, &binding, relations(&receipt))
        .ok()
        .unwrap();
    assert_same_entities(&first, &second);
}

fn relations(receipt: &PrismReceipt) -> &[crate::sketch_region::ArrangementRelation] {
    &receipt.diagnostic.as_ref().unwrap().relations
}

#[test]
fn missing_current_relation_is_not_recovered_from_old_chart_parents() {
    let (mut receipt, binding) = fixture();
    receipt.diagnostic.as_mut().unwrap().relations.remove(0);
    let error = capture_census(&receipt, &binding)
        .err()
        .expect("missing history rejected");
    assert_count(
        error,
        "current parent relation for directed use",
        0,
        1,
        true,
    );
}

#[test]
fn wrong_parent_kind_remains_typed_incomplete_census() {
    let (receipt, binding) = fixture();
    let mut contexts = body_contexts(&receipt.diagnostic.as_ref().unwrap().body)
        .ok()
        .unwrap();
    let mut vertex = contexts.pop().unwrap();
    vertex.ancestors.clear();
    let error = capture_census_from_contexts(&[vertex], &binding, relations(&receipt))
        .err()
        .unwrap();
    assert!(error.context.is_some());
    assert!(matches!(
        error.cause,
        ToleranceCensusCause::Kind {
            parent: None,
            actual: ShapeEnum::Vertex,
            ..
        }
    ));
}

#[test]
fn missing_body_remains_typed_incomplete_census() {
    let (mut receipt, binding) = fixture();
    receipt.diagnostic.take();
    assert_count(
        capture_census(&receipt, &binding).err().unwrap(),
        "actual prism body",
        0,
        1,
        false,
    );
}

fn assert_same_entities(first: &PrismToleranceCensus, second: &PrismToleranceCensus) {
    assert_eq!(first.entities.len(), second.entities.len());
    for row in &first.entities {
        let other = second
            .entities
            .iter()
            .find(|other| other.entity.same_physical(&row.entity))
            .unwrap();
        assert_eq!(row.contexts.len(), other.contexts.len());
        assert!(row.contexts.iter().all(|key| other.contexts.contains(key)));
    }
}

pub(super) fn assert_count(
    error: ToleranceCensusFailure,
    quantity: &str,
    actual: usize,
    expected: usize,
    attributed: bool,
) {
    assert_eq!(error.context.is_some(), attributed);
    let ToleranceCensusCause::Count {
        quantity: found,
        actual: got,
        expected: needed,
    } = error.cause
    else {
        panic!("expected typed census cardinality failure");
    };
    assert_eq!((found, got, needed), (quantity, actual, expected));
}

#[test]
fn owned_census_has_send_sync_coefficients_and_keys() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<PrismToleranceCensus>();
}
