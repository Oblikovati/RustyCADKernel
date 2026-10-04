use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};

/// Copies a replacement endpoint only when native safe-input mode requests isolation.
pub(super) fn safe_merge_vertex(vertex: &Vertex, safe_input_mode: bool) -> Result<Vertex, Failure> {
    if !safe_input_mode {
        return Ok(vertex.clone());
    }
    rustycad_tkbrep::topods::vertex(&vertex.empty_copied()?)
}

/// Records copied endpoints so replacement topology refers to the safe copies.
pub(super) fn record_safe_vertex(
    context: &Handle<ShapeBuildReShape>,
    vertex: &Vertex,
    replacement: &Vertex,
    safe_input_mode: bool,
) -> Result<(), Failure> {
    if !safe_input_mode {
        return Ok(());
    }
    replace_vertex(context, vertex, replacement)
}

fn replace_vertex(
    context: &Handle<ShapeBuildReShape>,
    vertex: &Vertex,
    replacement: &Vertex,
) -> Result<(), Failure> {
    let original: Shape = vertex.clone().into_shape();
    let copied: Shape = replacement.clone().into_shape();
    context.replace(&original, &copied)
}
