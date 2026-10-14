// SPDX-License-Identifier: GPL-2.0-only
//! Read physical endpoint identities while their actual DS still owns the shapes.
use super::{copy_inputs::CopiedSources, partition_types::*};
use crate::bop_ds::DataStructure;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, vertex},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture_vertices(
    copied: &CopiedSources,
    structure: &DataStructure,
    partial: &mut OwnedPartitionSnapshot,
    identity: fn(&Shape) -> PartitionOccurrenceIdentity,
) -> Result<(), PartitionReadFailure> {
    for index in 0..structure.nb_shapes() {
        let Some(shape) = observed_vertex_shape(index, structure)? else {
            continue;
        };
        partial
            .vertices
            .push(capture_vertex(index, shape, copied, structure, identity)?);
    }
    Ok(())
}
fn capture_vertex(
    index: i32,
    shape: &Shape,
    copied: &CopiedSources,
    structure: &DataStructure,
    identity: fn(&Shape) -> PartitionOccurrenceIdentity,
) -> Result<PartitionVertexObservation, PartitionReadFailure> {
    let original_parents = vertex_parents(index, copied, structure);
    let point = vertex_point(shape).map_err(|cause| {
        let mut failure = vertex_failure(index, cause);
        failure.parents = original_parents.clone();
        failure
    })?;
    Ok(PartitionVertexObservation {
        shape_index: index,
        occurrence: identity(shape),
        point,
        original_parents,
    })
}
fn vertex_point(shape: &Shape) -> Result<[f64; 3], Failure> {
    let point = BRepTool::pnt(&vertex(shape)?)?;
    Ok([point.x(), point.y(), point.z()])
}
fn vertex_parents(
    index: i32,
    copied: &CopiedSources,
    structure: &DataStructure,
) -> Vec<super::OriginalMemberIdentity> {
    copied
        .members
        .iter()
        .filter(|(_, shape)| structure.index(shape) == index)
        .map(|(parent, _)| parent.clone())
        .collect()
}
fn vertex_failure(index: i32, cause: Failure) -> PartitionReadFailure {
    let mut failure = super::partition_diagnostics::read_failure(
        None,
        None,
        PartitionReadCause::Native(Box::new(cause)),
    );
    failure.vertex_index = Some(index);
    failure
}

fn observed_vertex_shape(
    index: i32,
    structure: &DataStructure,
) -> Result<Option<&Shape>, PartitionReadFailure> {
    let shape = structure
        .shape(index)
        .map_err(|cause| vertex_failure(index, cause))?;
    let kind = shape
        .shape_type()
        .map_err(|cause| vertex_failure(index, cause))?;
    Ok((kind == ShapeEnum::Vertex).then_some(shape))
}
