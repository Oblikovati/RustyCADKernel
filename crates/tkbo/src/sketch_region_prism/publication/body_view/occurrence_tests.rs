// SPDX-License-Identifier: GPL-2.0-only
use super::{error::*, occurrences, replay, test_fixture::*};
use crate::sketch_region_prism::frozen::proof_join::JoinedVertexOperand;
pub(super) fn operands() -> (
    crate::sketch_region_prism::frozen::FrozenPrismUse,
    JoinedVertexOperand,
) {
    let row = symbolic_use(cap_endpoint(&names()));
    let vertex = JoinedVertexOperand {
        vertex: row.key.0.last().unwrap().0,
        occurrence: row.key.clone(),
        local_point: [1., -0., 0.],
        placement: placement(),
    };
    (row, vertex)
}
#[test]
fn vertex_view_keeps_local_one_and_huge_translation_separate() {
    let names = names();
    let (source, vertex) = operands();
    let row = occurrences::project(&source, &names, &[vertex]).unwrap();
    assert_eq!(
        row.local_vertex().unwrap().map(f64::to_bits),
        [1., -0., 0.].map(f64::to_bits)
    );
    assert_eq!(row.placement().origin[0], 9007199254740992.);
    assert_eq!(row.key(), names.occurrence(&source.key).unwrap());
    assert_eq!(row.orientation(), source.orientation);
    assert!(row.children().is_empty());
}
#[test]
fn raw_vertex_copy_isolated_from_source_mutation_and_drop() {
    let names = names();
    let (source, mut vertex) = operands();
    let row = occurrences::project(&source, &names, &[vertex.clone()]).unwrap();
    let mut before = Vec::new();
    replay::occurrence(&row, &mut before);
    vertex.local_point[0] = 99.;
    drop(vertex);
    drop(source);
    drop(names);
    let mut after = Vec::new();
    replay::occurrence(&row, &mut after);
    assert_eq!(before, after);
}
#[test]
fn missing_duplicate_or_different_owner_frame_is_typed_incomplete() {
    let names = names();
    let (source, mut vertex) = operands();
    assert!(occurrences::project(&source, &names, &[]).is_err());
    assert!(occurrences::project(&source, &names, &[vertex.clone(), vertex.clone()]).is_err());
    vertex.placement.origin[0] = 0.;
    let error = occurrences::project(&source, &names, &[vertex])
        .err()
        .unwrap();
    assert!(matches!(
        error.cause,
        PrismBodyProjectionCause::OperandMismatch { .. }
    ));
    assert!(matches!(
        error.entity,
        Some(PrismBodyProjectionEntity::Occurrence(_))
    ));
}
