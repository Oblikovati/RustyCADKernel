// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::{
    RegionContextObservation, RegionObservedContext, observe_region_contexts,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

fn context_fixture() -> (Vec<RegionObservedContext>, PrismHistoricalStageBindings) {
    let edge = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let RegionContextObservation::Captured(rows) = observe_region_contexts(&edge) else {
        panic!("actual context capture");
    };
    let names = names_for(&rows, PrismHistoryStage::InputCopy);
    (rows, names)
}
pub(super) fn names_for(
    rows: &[RegionObservedContext],
    stage: PrismHistoryStage,
) -> PrismHistoricalStageBindings {
    let members = vec![
        member_for(rows, ShapeEnum::Edge, 4101),
        member_for(rows, ShapeEnum::Vertex, 4102),
    ];
    let uses = rows
        .iter()
        .map(|row| PrismHistoricalUseBinding {
            actual: row.clone(),
            authored_use: semantic_context(row),
        })
        .collect();
    PrismHistoricalStageBindings {
        stage,
        members,
        uses,
    }
}
fn member_for(rows: &[RegionObservedContext], kind: ShapeEnum, name: u64) -> PrismSourceBinding {
    PrismSourceBinding {
        actual: rows
            .iter()
            .find(|r| r.kind == kind)
            .unwrap()
            .occurrence
            .clone(),
        member: AuthoredPrismId(name),
    }
}
fn semantic_context(row: &RegionObservedContext) -> AuthoredPrismId {
    AuthoredPrismId(match (row.kind, row.occurrence.orientation) {
        (ShapeEnum::Edge, _) => 4201,
        (ShapeEnum::Vertex, Orientation::Forward) => 4202,
        (ShapeEnum::Vertex, Orientation::Reversed) => 4203,
        unexpected => panic!("authored cubic role expected, got {unexpected:?}"),
    })
}
pub(super) fn bind<'a>(
    rows: &'a [RegionObservedContext],
    names: &PrismHistoricalStageBindings,
) -> HistoryCheck<BoundHistoricalContexts<'a>> {
    contexts::bind_historical_contexts(
        AuthoredPrismId(4001),
        names,
        rows,
        PrismHistoryLimits::bounded_v1(),
    )
}
#[test]
fn historical_contexts_keep_shared_vertex_and_distinct_authored_endpoint_uses() {
    let (rows, names) = context_fixture();
    let bound = bind(&rows, &names).unwrap();
    assert_eq!(bound.owned.len(), 3);
    let ends: Vec<_> = bound
        .owned
        .iter()
        .filter(|row| row.occurrence.member.kind == ShapeEnum::Vertex as u8)
        .collect();
    assert_eq!(ends[0].occurrence.member, ends[1].occurrence.member);
    assert_ne!(ends[0].key, ends[1].key);
    assert_eq!(
        bound
            .owned
            .iter()
            .find(|r| r.occurrence.member.kind == ShapeEnum::Edge as u8)
            .unwrap()
            .children
            .len(),
        2
    );
}
#[test]
fn historical_context_bindings_refuse_missing_duplicate_and_unused_authorship() {
    let (rows, mut names) = context_fixture();
    names.uses.pop();
    assert!(bind(&rows, &names).is_err());
    let (rows, mut names) = context_fixture();
    names.members.push(PrismSourceBinding {
        actual: names.members[0].actual.clone(),
        member: AuthoredPrismId(9999),
    });
    assert!(bind(&rows, &names).is_err());
    let (rows, mut names) = context_fixture();
    names.uses[1].actual = names.uses[0].actual.clone();
    assert!(bind(&rows, &names).is_err());
}
#[test]
fn historical_context_rows_reorder_without_changing_authored_identity_or_children() {
    let (mut rows, names) = context_fixture();
    let mut before = bind(&rows, &names).unwrap().owned;
    rows.reverse();
    let mut after = bind(&rows, &names).unwrap().owned;
    before.sort();
    after.sort();
    assert_eq!(before, after);
}
#[test]
fn historical_contexts_refuse_changed_ancestor_and_member_identity_collision() {
    let (mut rows, names) = context_fixture();
    let end = rows.iter_mut().find(|r| !r.ancestors.is_empty()).unwrap();
    end.ancestors[0] = end.occurrence.clone();
    assert!(bind(&rows, &names).is_err());
    let (rows, mut names) = context_fixture();
    names.members[1].member = names.members[0].member;
    assert!(bind(&rows, &names).is_err());
}
#[test]
fn owned_history_context_types_are_send_sync() {
    fn require<T: Send + Sync>() {}
    require::<PrismHistoryMemberKey>();
    require::<PrismHistoryUseKey>();
    require::<PrismHistoricalOccurrence>();
    require::<PrismHistoricalContext>();
    require::<PrismHistoryCause>();
    require::<PrismHistoricalStageBindings>();
}
