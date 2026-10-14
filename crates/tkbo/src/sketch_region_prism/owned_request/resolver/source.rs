// SPDX-License-Identifier: GPL-2.0-only
//! Native endpoint handles author roles before cumulative traversal is considered.
use super::super::construction::ConstructedPolynomialSource;
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge, vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, compose};
pub(super) fn original_contexts(
    prepared: &PreparedPolynomialPrism,
    actual: &[RegionObservedContext],
) -> ResolveCheck<Vec<NamedSourceContext>> {
    let source = require_local(prepared.retained.source.as_ref(), "constructor-held source")?;
    admit_original_source(prepared, source, actual)?;
    let root = PartitionOccurrenceIdentity::capture(&source.edge);
    let mut result = vec![named_context(
        actual,
        &root,
        &[],
        ShapeEnum::Edge,
        SemanticNameRole::Boundary,
        SemanticNameRole::BoundaryUse,
    )?];
    append_native_contexts(source, actual, &mut result)?;
    complete_context_coverage(&result, actual)?;
    Ok(result)
}
fn admit_original_source(
    prepared: &PreparedPolynomialPrism,
    source: &ConstructedPolynomialSource,
    actual: &[RegionObservedContext],
) -> ResolveCheck<()> {
    admit_constructor_handles(source)?;
    let limits = prepared.retained.request.policy.publication.history.limits;
    let current = capture_history_contexts(&source.edge, PrismHistoryStage::Original, limits)?;
    require_count(
        "current source exactly matches retained original context census",
        usize::from(current == actual),
        1,
    )
}
fn admit_constructor_handles(source: &ConstructedPolynomialSource) -> ResolveCheck<()> {
    let same = source.native_edge.is_same(&source.edge)
        && source.native_start.is_same(&source.closure)
        && source.native_end.is_same(&source.closure);
    require_count(
        "constructor-held native edge and shared closure definitions",
        usize::from(same),
        1,
    )?;
    verify_native_endpoint(&source.native_edge, &source.native_start, 0.)?;
    verify_native_endpoint(&source.native_edge, &source.native_end, 1.)
}
fn append_native_contexts(
    source: &ConstructedPolynomialSource,
    actual: &[RegionObservedContext],
    result: &mut Vec<NamedSourceContext>,
) -> ResolveCheck<()> {
    for (native, role) in [
        (&source.native_start, SemanticNameRole::StartUse),
        (&source.native_end, SemanticNameRole::EndUse),
    ] {
        result.push(native_context(source, actual, native, role)?);
    }
    Ok(())
}
fn native_context(
    source: &ConstructedPolynomialSource,
    actual: &[RegionObservedContext],
    native: &Shape,
    role: SemanticNameRole,
) -> ResolveCheck<NamedSourceContext> {
    let root = PartitionOccurrenceIdentity::capture(&source.edge);
    let orientation = compose(source.edge.orientation(), native.orientation());
    let endpoint = PartitionOccurrenceIdentity::capture(&native.oriented(orientation));
    named_context(
        actual,
        &endpoint,
        &[root],
        ShapeEnum::Vertex,
        SemanticNameRole::ClosureVertex,
        role,
    )
}
fn named_context(
    actual: &[RegionObservedContext],
    occurrence: &PartitionOccurrenceIdentity,
    ancestors: &[PartitionOccurrenceIdentity],
    kind: ShapeEnum,
    member: SemanticNameRole,
    directed_use: SemanticNameRole,
) -> ResolveCheck<NamedSourceContext> {
    let found: Vec<_> = actual
        .iter()
        .filter(|row| matches_native_context(row, occurrence, ancestors, kind))
        .collect();
    require_count("one constructor-held native semantic use", found.len(), 1)?;
    Ok(NamedSourceContext {
        actual: found[0].clone(),
        member,
        directed_use,
    })
}
fn matches_native_context(
    row: &RegionObservedContext,
    occurrence: &PartitionOccurrenceIdentity,
    ancestors: &[PartitionOccurrenceIdentity],
    kind: ShapeEnum,
) -> bool {
    &row.occurrence == occurrence && row.ancestors == ancestors && row.kind == kind
}
fn verify_native_endpoint(edge_shape: &Shape, endpoint: &Shape, expected: f64) -> ResolveCheck<()> {
    let actual = native_parameter(edge_shape, endpoint).map_err(|cause| {
        ResolutionCause::History(Box::new(PrismHistoryCause::Native(std::sync::Arc::new(
            cause,
        ))))
    })?;
    if actual != expected {
        return Err(ResolutionCause::NativeParameter { actual, expected });
    }
    Ok(())
}
fn native_parameter(edge_shape: &Shape, endpoint: &Shape) -> Result<f64, Failure> {
    BRepTool::parameter(&vertex(endpoint)?, &edge(edge_shape)?)
}
