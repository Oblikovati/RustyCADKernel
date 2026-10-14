// SPDX-License-Identifier: GPL-2.0-only
use super::context_tests::{bind, names_for};
use super::*;
use crate::sketch_region::{
    RegionCopyContextObservation, RegionCopyContexts, observe_copy_contexts,
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};

fn copy_fixture() -> RegionCopyContexts {
    let source = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let mut copier = BRepBuilderAPICopy::with_shape(&source, true, true).unwrap();
    let copied = copier.shape().unwrap().clone();
    let RegionCopyContextObservation::Captured(rows) =
        observe_copy_contexts(&source, &copied, &mut copier)
    else {
        panic!("actual copier contexts expected");
    };
    rows
}
fn project(rows: &RegionCopyContexts) -> HistoryCheck<Vec<PrismHistoricalCopyQuery>> {
    let source_names = names_for(&rows.source, PrismHistoryStage::Original);
    let copied_names = names_for(&rows.copied, PrismHistoryStage::InputCopy);
    let source = bind(&rows.source, &source_names)?;
    let copied = bind(&rows.copied, &copied_names)?;
    copied_contexts::project_copier_queries(rows, &source, &copied)
}
#[test]
fn actual_directed_copier_queries_project_unique_ancestry_and_native_images() {
    let rows = copy_fixture();
    let result = project(&rows).unwrap();
    assert_eq!(result.len(), 3);
    assert!(result.iter().all(|row| row.native_images.len() == 1));
    assert!(
        result
            .iter()
            .all(|row| row.source.0.last().unwrap().0.stage == PrismHistoryStage::Original)
    );
    assert!(
        result
            .iter()
            .all(|row| row.copied.0.last().unwrap().0.stage == PrismHistoryStage::InputCopy)
    );
    fn require<T: Send + Sync>() {}
    require::<PrismHistoricalCopyQuery>();
}
#[test]
fn directed_copier_projection_refuses_missing_duplicate_and_empty_query_answers() {
    let mut rows = copy_fixture();
    rows.queries.pop();
    assert!(project(&rows).is_err());
    let mut rows = copy_fixture();
    rows.queries[1] = rows.queries[0].clone();
    assert!(project(&rows).is_err());
    let mut rows = copy_fixture();
    rows.queries[0].images = Ok(Vec::new());
    assert!(project(&rows).is_err());
}
#[test]
fn directed_copier_projection_preserves_query_error_and_wrong_image_refusal() {
    let mut rows = copy_fixture();
    let expected = std::sync::Arc::new(rustycad_tkernel::standard::Failure::new(
        "recorded test query failure",
    ));
    rows.queries[0].images = Err(expected.clone());
    let Err(PrismHistoryCause::AtUse { entity, cause }) = project(&rows) else {
        panic!("native query failure expected");
    };
    assert_eq!(
        entity.0.last().unwrap().0.stage,
        PrismHistoryStage::Original
    );
    let PrismHistoryCause::Native(cause) = *cause else {
        panic!("preserved native cause expected");
    };
    assert_eq!(cause.message(), expected.message());
    let mut rows = copy_fixture();
    rows.queries[0].images = Ok(vec![rows.source[0].occurrence.clone()]);
    assert!(project(&rows).is_err());
}
