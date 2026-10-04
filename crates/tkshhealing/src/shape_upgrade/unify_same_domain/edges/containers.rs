use super::super::UnifyFields;
use super::{EdgeMergeOptions, merge_planar_face_edges};
use crate::shape_build::ShapeBuildReShape;
use crate::shape_upgrade::unify_same_domain::edges::rebuild_planar_face_pcurves;
use crate::shape_upgrade::unify_same_domain::face_wires;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;

/// Merges face boundaries under one reshape context for the complete input.
pub(super) fn merge_nested_children(
    shape: &Shape,
    fields: &UnifyFields,
    context: &Handle<ShapeBuildReShape>,
) -> Result<Shape, Failure> {
    let faces = unique_faces(shape)?;
    let options = EdgeMergeOptions {
        angular_tolerance: fields.angular_tolerance,
        concat_bspline_edges: fields._concat_bspline_edges,
        keep_shapes: &fields.keep_shapes,
        safe_input_mode: fields.safe_input_mode,
        context,
    };
    for face in faces {
        merge_face_in_context(&face, &options)?;
    }
    context.apply(shape, ShapeEnum::Shape)
}

fn unique_faces(shape: &Shape) -> Result<Vec<Face>, Failure> {
    let mut unique = Vec::new();
    for candidate in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = rustycad_tkbrep::topods::face(&candidate)?;
        if !unique.iter().any(|known: &Face| known.is_same(&face)) {
            unique.push(face);
        }
    }
    Ok(unique)
}

fn merge_face_in_context(original: &Face, options: &EdgeMergeOptions<'_>) -> Result<(), Failure> {
    let context = options.context;
    let applied = context.apply(&original.clone().into_shape(), ShapeEnum::Shape)?;
    let mut replacement = merge_planar_face_edges(&applied, options)?;
    if !applied.is_same(&original.clone().into_shape()) && replacement.is_same(&applied) {
        replacement = rebuild_pcurves_if_planar(&replacement)?;
    }
    if !replacement.is_same(&original.clone().into_shape()) {
        context.replace(&original.clone().into_shape(), &replacement)?;
    }
    Ok(())
}

fn rebuild_pcurves_if_planar(shape: &Shape) -> Result<Shape, Failure> {
    let face = rustycad_tkbrep::topods::face(shape)?;
    if BRepAdaptorSurface::with_face(&face, false)?.get_type()? != SurfaceType::Plane {
        return Ok(shape.clone());
    }
    if face_wires(&face)?.len() != 1 {
        return Ok(shape.clone());
    }
    rebuild_planar_face_pcurves(shape)
}
