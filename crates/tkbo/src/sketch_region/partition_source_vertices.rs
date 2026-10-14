// SPDX-License-Identifier: GPL-2.0-only
//! Unsplit and closed sources retain actual child occurrences and mapped parameters.
use super::{
    partition_diagnostics::{native_failure, read_failure},
    partition_types::*,
};
use crate::bop_ds::DataStructure;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, ShapeIterator, edge, vertex},
};

pub(super) fn capture_source_vertices(
    index: i32,
    source: &Shape,
    structure: &DataStructure,
    identity: fn(&Shape) -> PartitionOccurrenceIdentity,
) -> Result<Vec<SourceVertexObservation>, PartitionReadFailure> {
    let typed = edge(source).map_err(|cause| native_failure(index, cause))?;
    let mut children = ShapeIterator::new(source, true, true);
    let mut vertices = Vec::new();
    while children.more() {
        let child = children
            .value()
            .map_err(|cause| native_failure(index, cause))?;
        let parameter = source_parameter(index, child, &typed)?;
        vertices.push(source_vertex(index, child, parameter, structure, identity)?);
        children.next();
    }
    Ok(vertices)
}
fn source_vertex(
    source: i32,
    child: &Shape,
    parameter: f64,
    structure: &DataStructure,
    identity: fn(&Shape) -> PartitionOccurrenceIdentity,
) -> Result<SourceVertexObservation, PartitionReadFailure> {
    let index = structure.index(child);
    if index < 0 {
        return Err(missing_vertex(source, index));
    }
    Ok(SourceVertexObservation {
        shape_index: index,
        occurrence: identity(child),
        parameter,
    })
}

fn source_parameter(
    index: i32,
    child: &Shape,
    source: &rustycad_tkbrep::topods::Edge,
) -> Result<f64, PartitionReadFailure> {
    let typed = vertex(child).map_err(|cause| native_failure(index, cause))?;
    BRepTool::parameter(&typed, source).map_err(|cause| native_failure(index, cause))
}

fn missing_vertex(source: i32, index: i32) -> PartitionReadFailure {
    read_failure(
        Some(source),
        None,
        PartitionReadCause::MissingVertexIndex(index),
    )
}
