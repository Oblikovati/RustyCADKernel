use super::super::UnifyFields;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// Rebuilds a single-face compound after its contained wire changes.
pub(super) fn merge_nested_children(shape: &Shape, fields: &UnifyFields) -> Result<Shape, Failure> {
    if shape.shape_type()? != ShapeEnum::Compound {
        return Ok(shape.clone());
    }
    // Native merges edge chains globally; per-face edits can disconnect shared shell edges.
    if count_faces(shape)? > 1 {
        return Ok(shape.clone());
    }
    let context = Handle::new(ShapeBuildReShape::new());
    context.set_mode_consider_location(true);
    if !replace_changed_children(shape, fields, &context)? {
        return Ok(shape.clone());
    }
    context.apply(shape, ShapeEnum::Shape)
}

fn count_faces(shape: &Shape) -> Result<usize, Failure> {
    Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).map(Iterator::count)
}

/// Applies supported edge merges to each direct child and records only changed topology.
fn replace_changed_children(
    shape: &Shape,
    fields: &UnifyFields,
    context: &Handle<ShapeBuildReShape>,
) -> Result<bool, Failure> {
    let mut changed = false;
    for child in ShapeIterator::new(shape, true, true) {
        let replacement = merge_nested_shape(&child, fields)?;
        if !replacement.is_same(&child) {
            context.replace(&child, &replacement)?;
            changed = true;
        }
    }
    Ok(changed)
}

/// Dispatches nested wires and faces before descending through shape containers.
fn merge_nested_shape(shape: &Shape, fields: &UnifyFields) -> Result<Shape, Failure> {
    match shape.shape_type()? {
        ShapeEnum::Wire => merge_wire(shape, fields),
        ShapeEnum::Face => merge_face(shape, fields),
        ShapeEnum::Compound => merge_nested_children(shape, fields),
        _ => Ok(shape.clone()),
    }
}

fn merge_wire(shape: &Shape, fields: &UnifyFields) -> Result<Shape, Failure> {
    super::merge_wire_chains(
        shape,
        fields.angular_tolerance,
        true,
        fields._concat_bspline_edges,
        &fields.keep_shapes,
        fields.safe_input_mode,
    )
}

fn merge_face(shape: &Shape, fields: &UnifyFields) -> Result<Shape, Failure> {
    super::merge_planar_face_edges(
        shape,
        fields.angular_tolerance,
        fields._concat_bspline_edges,
        &fields.keep_shapes,
        fields.safe_input_mode,
    )
}
