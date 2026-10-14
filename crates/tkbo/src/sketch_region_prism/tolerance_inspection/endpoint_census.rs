// SPDX-License-Identifier: GPL-2.0-only
//! Each physical endpoint parameter has one owned operand set and every directed context.
use super::super::{PrismBodyContext, region_certificate::current_binding::CurrentPrismBinding};
use super::{census_contexts::*, census_types::*, endpoint_contexts::*};
use crate::sketch_region::PartitionOccurrenceIdentity;
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Vertex},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

pub(super) fn capture_endpoints(
    contexts: &[PrismBodyContext],
    entities: &[ToleranceEntityCensus],
    binding: &CurrentPrismBinding,
) -> CensusCheck<Vec<ToleranceEndpointCensus>> {
    check_vertex_use_counts(contexts)?;
    let mut endpoints = Vec::new();
    for entity in entities.iter().filter(|row| row.kind == ShapeEnum::Edge) {
        let captured = append_edge_endpoints(contexts, entity, binding, &mut endpoints);
        attach_context(captured, &entity.contexts[0])?;
    }
    complete_endpoint_uses(contexts, &endpoints)?;
    Ok(endpoints)
}

fn append_edge_endpoints(
    contexts: &[PrismBodyContext],
    entity: &ToleranceEntityCensus,
    binding: &CurrentPrismBinding,
    endpoints: &mut Vec<ToleranceEndpointCensus>,
) -> CensusCheck<()> {
    let actual = physical_edge(contexts, &entity.entity)?;
    require_directed(actual.orientation())?;
    let edge = Edge::unchecked(actual.oriented(Orientation::Forward));
    let space = bound_space(binding, &entity.entity)?;
    for vertex in canonical_vertices(&edge)? {
        let row = capture_endpoint(contexts, &edge, &vertex, space)?;
        endpoints.push(row);
    }
    Ok(())
}

fn capture_endpoint(
    contexts: &[PrismBodyContext],
    edge: &Edge,
    vertex: &Vertex,
    space: &CapturedSpaceRepresentation,
) -> CensusCheck<ToleranceEndpointCensus> {
    let uses = endpoint_uses(contexts, edge, vertex)?;
    let operands = capture_endpoint_operands(edge, vertex, space, uses.clone());
    attach_context(operands, &uses[0])
}

fn capture_endpoint_operands(
    edge: &Edge,
    vertex: &Vertex,
    space: &CapturedSpaceRepresentation,
    contexts: Vec<ToleranceUseContext>,
) -> CensusCheck<ToleranceEndpointCensus> {
    let point = BRepTool::pnt(vertex)?;
    Ok(ToleranceEndpointCensus {
        edge: PartitionOccurrenceIdentity::capture(edge),
        vertex: PartitionOccurrenceIdentity::capture(vertex),
        parameter: BRepTool::parameter(vertex, edge)?,
        point: [point.x(), point.y(), point.z()],
        space: space.clone(),
        contexts,
    })
}

fn bound_space<'a>(
    binding: &'a CurrentPrismBinding,
    edge: &PartitionOccurrenceIdentity,
) -> CensusCheck<&'a CapturedSpaceRepresentation> {
    // Every matching trim was freshly proved from this same physical curve and is rebound at Commit.
    let found = binding
        .charts
        .iter()
        .flat_map(|chart| &chart.captured.trims)
        .find(|trim| trim.output.same_physical(edge));
    count(
        "bound actual endpoint curve",
        usize::from(found.is_some()),
        1,
    )?;
    Ok(&found.unwrap().space)
}

fn complete_endpoint_uses(
    contexts: &[PrismBodyContext],
    endpoints: &[ToleranceEndpointCensus],
) -> CensusCheck<()> {
    for context in contexts {
        if context.occurrence.shape_type()? != ShapeEnum::Vertex {
            continue;
        }
        let key = capture_context(context);
        let uses = endpoints
            .iter()
            .flat_map(|row| &row.contexts)
            .filter(|found| **found == key)
            .count();
        attach_context(count("endpoint operand set per vertex use", uses, 1), &key)?;
    }
    Ok(())
}
