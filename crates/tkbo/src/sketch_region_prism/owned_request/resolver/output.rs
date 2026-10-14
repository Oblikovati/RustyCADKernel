// SPDX-License-Identifier: GPL-2.0-only
//! Output use names come from exact chart/trim roles and source endpoint anchors, never traversal rank.
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_prism::region_certificate::current_binding::*;
use crate::sketch_region_prism::{PrismBodyContext, frozen::PrismDirectedBinding};
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) fn directed_outputs(
    prepared: &PreparedPolynomialPrism,
    plan: &SingleBoundaryNamePlan,
    binding: &CurrentPrismBinding,
) -> ResolveCheck<Vec<PrismDirectedBinding>> {
    let actual = actual_body_contexts(prepared)?;
    require_count(
        "complete one-Jordan prism directed output universe",
        actual.len(),
        26,
    )?;
    actual
        .iter()
        .map(|row| output_binding(plan, binding, row))
        .collect()
}
fn output_binding(
    plan: &SingleBoundaryNamePlan,
    binding: &CurrentPrismBinding,
    actual: &PrismBodyContext,
) -> ResolveCheck<PrismDirectedBinding> {
    let role = output_role(binding, actual)?;
    Ok(PrismDirectedBinding {
        actual: actual.clone(),
        authored_use: materialize::authored(plan, SemanticNameStage::Output, role)?,
    })
}
fn output_role(
    binding: &CurrentPrismBinding,
    actual: &PrismBodyContext,
) -> ResolveCheck<SemanticNameRole> {
    use SemanticNameRole::*;
    let kind = actual
        .occurrence
        .shape_type()
        .map_err(ResolutionCause::Native)?;
    match kind {
        ShapeEnum::Solid if actual.ancestors.is_empty() => Ok(SolidUse),
        ShapeEnum::Shell if actual.ancestors.len() == 1 => Ok(ShellUse),
        ShapeEnum::Face | ShapeEnum::Wire => face_or_loop(binding, actual, kind),
        ShapeEnum::Edge | ShapeEnum::Vertex => output_trims::role(binding, actual, kind),
        _ => Err(ResolutionCause::Missing {
            required: "certified solid/shell/face/wire/trim/endpoint context",
        }),
    }
}
fn face_or_loop(
    binding: &CurrentPrismBinding,
    actual: &PrismBodyContext,
    kind: ShapeEnum,
) -> ResolveCheck<SemanticNameRole> {
    admit_chart_trims(chart_for(binding, actual)?)?;
    let depth = if kind == ShapeEnum::Face { 2 } else { 3 };
    require_count(
        "face or sole wire under certified prism ancestry",
        actual.ancestors.len(),
        depth,
    )?;
    Ok(if kind == ShapeEnum::Face {
        SemanticNameRole::FaceUse
    } else {
        SemanticNameRole::LoopUse
    })
}
pub(super) fn chart_for<'a>(
    binding: &'a CurrentPrismBinding,
    actual: &PrismBodyContext,
) -> ResolveCheck<&'a BoundPrismChart> {
    let face = actual_face_identity(actual)?;
    let found: Vec<_> = binding
        .charts
        .iter()
        .filter(|chart| chart.output == face)
        .collect();
    require_count("one exact certified chart for output face", found.len(), 1)?;
    Ok(found[0])
}

fn actual_body_contexts(prepared: &PreparedPolynomialPrism) -> ResolveCheck<Vec<PrismBodyContext>> {
    let body = &require_local(
        current::receipt(prepared)?.diagnostic.as_ref(),
        "actual swept body",
    )?
    .body;
    preflight_history_root(
        body,
        prepared.retained.request.policy.publication.history.limits,
    )
    .map_err(|cause| ResolutionCause::History(Box::new(PrismHistoryCause::Bounds(cause))))?;
    let actual = crate::sketch_region_prism::contexts::body_contexts(body)
        .map_err(|cause| ResolutionCause::Prism(Box::new(cause)))?;
    Ok(actual)
}

fn admit_chart_trims(chart: &BoundPrismChart) -> ResolveCheck<()> {
    require_count(
        "certified chart has admitted trims",
        chart.captured.trims.len(),
        if chart.role == crate::sketch_region_prism::PrismImageRole::Generated {
            4
        } else {
            1
        },
    )?;
    Ok(())
}

fn actual_face_identity(actual: &PrismBodyContext) -> ResolveCheck<PartitionOccurrenceIdentity> {
    let mut faces = Vec::new();
    for shape in actual.ancestors.iter().chain([&actual.occurrence]) {
        if shape.shape_type().map_err(ResolutionCause::Native)? == ShapeEnum::Face {
            faces.push(PartitionOccurrenceIdentity::capture(shape));
        }
    }
    require_count(
        "one actual face in complete output ancestry",
        faces.len(),
        1,
    )?;
    Ok(faces.remove(0))
}
