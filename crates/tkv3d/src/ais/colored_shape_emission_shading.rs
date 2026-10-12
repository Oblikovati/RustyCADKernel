//! Collect each native shaded stage from the live Value after preceding driver callbacks.
use super::{
    AisColoredCompoundRef, AisColoredShapeSource, colored_shape_emission::ColoredShapeEmission,
};
use crate::{prs3d::Prs3dDrawer, std_prs::StdPrsShadedShape};
use rustycad_tkbrep::topods::TopoDSShapeReadSource;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;
pub(super) fn emit_colored_shading<S: AisColoredShapeSource + ?Sized>(
    reference: &AisColoredCompoundRef<'_>,
    drawer: &Handle<Prs3dDrawer>,
    closed: bool,
    emission: &mut ColoredShapeEmission<'_, S>,
) -> Result<(), Failure> {
    let triangles = collect_colored_triangles(reference, drawer, emission.object)?;
    if !triangles.is_null() {
        let group = colored_shaded_group(closed, emission)?;
        group.set_primitives_aspect(
            &drawer
                .shading_aspect()
                .aspect_ref()
                .observe()
                .observe_as::<dyn Graphic3dAspectsSource>(),
        );
        group.add_primitive_array(&triangles.cast(), true);
    }
    emit_colored_face_boundaries(reference, drawer, emission)
}
fn collect_colored_triangles<S: AisColoredShapeSource + ?Sized>(
    reference: &AisColoredCompoundRef<'_>,
    drawer: &Prs3dDrawer,
    object: &S,
) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
    let aspect = drawer.shading_aspect().aspect_ref().observe();
    let textured = aspect.to_map_texture() && !aspect.texture_map().is_null();
    let fields = object.shape_base().fields.read();
    let (origin, repeat, scale) = (fields.uv_origin, fields.uv_repeat, fields.uv_scale);
    drop(fields);
    reference.read_current_shape(|shape| {
        StdPrsShadedShape::fill_triangles_with_texels(shape, textured, origin, repeat, scale)
    })
}
fn colored_shaded_group<'s, S: AisColoredShapeSource + ?Sized>(
    closed: bool,
    emission: &'s mut ColoredShapeEmission<'_, S>,
) -> Result<&'s Handle<dyn Graphic3dGroup>, Failure> {
    let slot = if closed {
        &mut emission.closed
    } else {
        &mut emission.opened
    };
    if slot.is_null() {
        *slot = emission.prs.new_group()?;
        slot.set_closed(closed);
    }
    Ok(slot)
}
fn emit_colored_face_boundaries<S: AisColoredShapeSource + ?Sized>(
    reference: &AisColoredCompoundRef<'_>,
    drawer: &Prs3dDrawer,
    emission: &mut ColoredShapeEmission<'_, S>,
) -> Result<(), Failure> {
    if !drawer.face_boundary_draw() {
        return Ok(());
    }
    let segments = reference.read_current_shape(|shape| {
        StdPrsShadedShape::fill_face_boundaries_with_continuity(
            shape,
            drawer.face_boundary_upper_continuity(),
        )
    })?;
    if segments.is_null() {
        return Ok(());
    }
    emit_colored_boundary_array(drawer, &segments, emission)
}
fn emit_colored_boundary_array<S: AisColoredShapeSource + ?Sized>(
    drawer: &Prs3dDrawer,
    segments: &Handle<Graphic3dArrayOfSegments>,
    emission: &mut ColoredShapeEmission<'_, S>,
) -> Result<(), Failure> {
    if emission.boundary.is_null() {
        emission.boundary = emission.prs.new_group()?;
    }
    let line = drawer.face_boundary_aspect();
    let aspect = line
        .aspect_ref()
        .observe()
        .observe_as::<dyn Graphic3dAspectsSource>();
    emission.boundary.set_primitives_aspect(&aspect);
    emission
        .boundary
        .add_primitive_array(&segments.observe_as::<Graphic3dArrayOfPrimitives>(), true);
    Ok(())
}
