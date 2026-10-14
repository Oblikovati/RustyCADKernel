// SPDX-License-Identifier: GPL-2.0-only
//! Seam endpoints inherit their source t0/t1 anchor at both sweep ends; sweep parameter is separate.
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_prism::region_certificate::current_binding::CurrentPrismBinding;
use crate::sketch_region_prism::{
    PrismBodyContext,
    surface_charts::{CapturedPrismTrim, PrismTrimRole},
};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{edge, vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
pub(super) fn role(
    binding: &CurrentPrismBinding,
    actual: &PrismBodyContext,
    kind: ShapeEnum,
) -> ResolveCheck<SemanticNameRole> {
    let chart = output::chart_for(binding, actual)?;
    let trim = actual_trim(&chart.captured.trims, actual, kind)?;
    if kind == ShapeEnum::Edge {
        return Ok(edge_role(trim.role));
    }
    vertex_role(trim, actual)
}
fn actual_trim<'a>(
    trims: &'a [CapturedPrismTrim],
    actual: &PrismBodyContext,
    kind: ShapeEnum,
) -> ResolveCheck<&'a CapturedPrismTrim> {
    let source = actual_trim_edge(actual, kind)?;
    let identity = PartitionOccurrenceIdentity::capture(source);
    let found: Vec<_> = trims
        .iter()
        .filter(|trim| trim.output == identity)
        .collect();
    require_count(
        "one exact trim in actual certified face ancestry",
        found.len(),
        1,
    )?;
    Ok(found[0])
}
fn edge_role(role: PrismTrimRole) -> SemanticNameRole {
    match role {
        PrismTrimRole::First => SemanticNameRole::FirstRimUse,
        PrismTrimRole::Last => SemanticNameRole::LastRimUse,
        PrismTrimRole::Start => SemanticNameRole::SeamStartUse,
        PrismTrimRole::End => SemanticNameRole::SeamEndUse,
    }
}
fn vertex_role(
    trim: &CapturedPrismTrim,
    actual: &PrismBodyContext,
) -> ResolveCheck<SemanticNameRole> {
    match trim.role {
        PrismTrimRole::Start => Ok(SemanticNameRole::StartUse),
        PrismTrimRole::End => Ok(SemanticNameRole::EndUse),
        PrismTrimRole::First | PrismTrimRole::Last => native_rim_endpoint(trim, actual),
    }
}
fn native_rim_endpoint(
    trim: &CapturedPrismTrim,
    actual: &PrismBodyContext,
) -> ResolveCheck<SemanticNameRole> {
    let parameter = native_vertex_parameter(actual)?;
    require_count(
        "source-parameter-preserving rim interval",
        usize::from(trim.space.interval == [0., 1.]),
        1,
    )?;
    match parameter {
        0. => Ok(SemanticNameRole::StartUse),
        1. => Ok(SemanticNameRole::EndUse),
        actual => Err(ResolutionCause::EndpointParameter {
            actual,
            expected: [0., 1.],
        }),
    }
}

fn actual_trim_edge(
    actual: &PrismBodyContext,
    kind: ShapeEnum,
) -> ResolveCheck<&rustycad_tkbrep::topods::Shape> {
    let source = if kind == ShapeEnum::Edge {
        &actual.occurrence
    } else {
        require_local(
            actual.ancestors.last(),
            "vertex has actual parent trim edge",
        )?
    };
    require_count(
        "actual direct trim edge parent",
        usize::from(source.shape_type().map_err(ResolutionCause::Native)? == ShapeEnum::Edge),
        1,
    )?;
    Ok(source)
}

fn native_vertex_parameter(actual: &PrismBodyContext) -> ResolveCheck<f64> {
    let parent = require_local(actual.ancestors.last(), "rim endpoint actual parent")?;
    let parameter = BRepTool::parameter(
        &vertex(&actual.occurrence).map_err(ResolutionCause::Native)?,
        &edge(parent).map_err(ResolutionCause::Native)?,
    )
    .map_err(ResolutionCause::Native)?;
    Ok(parameter)
}
