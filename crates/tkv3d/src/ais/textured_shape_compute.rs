//! Native textured Compute sequencing preserves partially emitted shading before WF fallback.
use super::shape_bounds::shape_presentation_bounds_snapshot;
use super::{AisTexturedShapeSource, ais_textured_shape_update_attributes};
use crate::{
    prs_mgr::PrsMgrPresentableObjectMembers,
    prs3d::Prs3dBndBox,
    std_prs::{StdPrsShadedShape, StdPrsToolTriangulatedShape, StdPrsVolume, StdPrsWfShape},
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle, catch_native_failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point2;
use rustycad_tkservice::graphic3d::*;

/// Compute native display modes0..3 without clearing prior groups; `ais_textured_shape_compute(&object,&prs,3)?`.
pub fn ais_textured_shape_compute<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    let shape = object.shape_base().fields.read().shape.clone();
    if shape.is_null() {
        return Ok(());
    }
    if shape.shape_type()? == ShapeEnum::Compound && shape.nb_children() == 0 {
        return Ok(());
    }
    prepare_textured_shape_presentation(object, prs, &shape)?;
    match mode {
        0 => compute_textured_wireframe(object, prs, &shape),
        1 | 3 => compute_textured_shading(object, prs, &shape, mode),
        2 => compute_textured_bounds(object, prs, &shape),
        _ => Ok(()),
    }
}
fn prepare_textured_shape_presentation<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
) -> Result<(), Failure> {
    let priority = match shape.shape_type()? {
        ShapeEnum::Wire => Some(Graphic3dDisplayPriority::Above1),
        ShapeEnum::Edge => Some(Graphic3dDisplayPriority::Above2),
        ShapeEnum::Vertex => Some(Graphic3dDisplayPriority::Highlight),
        _ => None,
    };
    if let Some(priority) = priority {
        prs.set_visual(Graphic3dTypeOfStructure::All);
        prs.set_display_priority(priority)?;
    }
    if object.is_infinite() {
        prs.set_infinite_state(true);
    }
    Ok(())
}
fn compute_textured_wireframe<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    StdPrsToolTriangulatedShape::clear_on_own_deflection_change(shape, &drawer, true)?;
    StdPrsWfShape::add(prs, shape, &drawer)
}
fn compute_textured_shading<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
    mode: i32,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    StdPrsToolTriangulatedShape::clear_on_own_deflection_change(shape, &drawer, true)?;
    if shape.shape_type()? > ShapeEnum::Face || object.is_infinite() {
        return StdPrsWfShape::add(prs, shape, &drawer);
    }
    // OCCT 3d097a0 catches Standard_Failure from void/bool virtual adapters as well as checked calls.
    let shaded = catch_native_failure(|| add_textured_shading_geometry(object, prs, shape, mode));
    if shaded.is_err() {
        // Native catches Standard_Failure from both Add and updateAttributes; keeps existing groups.
        return StdPrsWfShape::add(prs, shape, &drawer);
    }
    Ok(())
}
fn add_textured_shading_geometry<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
    mode: i32,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    if mode == 1 {
        return add_plain_shaded_geometry(prs, shape, &drawer);
    }
    let (origin, repeat, scale) = textured_effective_uv_parameters(object);
    add_native_textured_geometry(prs, shape, &drawer, (origin, repeat, scale))?;
    ais_textured_shape_update_attributes(object, prs)
}
fn textured_effective_uv_parameters<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
) -> (Point2, Point2, Point2) {
    let texture = object.textured_shape_base().fields.read();
    let shape = object.shape_base().fields.read();
    let origin = if texture.custom_origin {
        shape.uv_origin
    } else {
        Point2::default()
    };
    let scale = if texture.scale {
        shape.uv_scale
    } else {
        Point2::new(1., 1.)
    };
    // Native myToRepeat is never consulted here or when constructing the texture sampler.
    (origin, shape.uv_repeat, scale)
}
fn compute_textured_bounds<S: AisTexturedShapeSource + ?Sized>(
    object: &S,
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
) -> Result<(), Failure> {
    let drawer = object.attributes();
    if object.is_infinite() {
        return StdPrsWfShape::add(prs, shape, &drawer);
    }
    Prs3dBndBox::add_box(prs, &shape_presentation_bounds_snapshot(object)?, &drawer)
}

fn add_plain_shaded_geometry(
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
    drawer: &Handle<crate::prs3d::Prs3dDrawer>,
) -> Result<(), Failure> {
    StdPrsShadedShape::add(
        prs,
        shape,
        drawer,
        StdPrsVolume::Autodetection,
        &Handle::default(),
    )
}
fn add_native_textured_geometry(
    prs: &Handle<dyn Graphic3dStructureSource>,
    shape: &Shape,
    drawer: &Handle<crate::prs3d::Prs3dDrawer>,
    uv: (Point2, Point2, Point2),
) -> Result<(), Failure> {
    let (origin, repeat, scale) = uv;
    StdPrsShadedShape::add_with_texels(
        prs,
        shape,
        drawer,
        true,
        origin,
        repeat,
        scale,
        StdPrsVolume::Autodetection,
        &Handle::default(),
    )
}
