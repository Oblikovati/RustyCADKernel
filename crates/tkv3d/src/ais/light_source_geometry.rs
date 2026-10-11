//! Native geometric light presentations; positions/directions live in the object's transform.
use super::{AisLightSourceMembers, AisLightSourceSource};
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::*};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::*;
use rustycad_tkservice::graphic3d::*;
type Presentation = Handle<dyn Graphic3dStructureSource>;
type Array = Handle<Graphic3dArrayOfPrimitives>;
pub(super) fn light_group(
    prs: &Presentation,
    aspect: Handle<dyn Graphic3dAspectsSource>,
    array: &Array,
    closed: bool,
) -> Result<(), Failure> {
    let group = prs.new_group()?;
    group.set_closed(closed);
    group.set_group_primitives_aspect(&aspect);
    group.add_primitive_array(array, true);
    Ok(())
}
pub(super) fn light_point(
    prs: &Presentation,
    aspect: Handle<dyn Graphic3dAspectsSource>,
    point: Point3,
) -> Result<(), Failure> {
    let points = Handle::new(Graphic3dArrayOfPoints::new(1, GRAPHIC3D_ARRAY_FLAGS_NONE)?);
    points.add_vertex_pnt(&point)?;
    light_group(prs, aspect, &points.cast(), false)
}
pub(super) fn light_sphere<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Presentation,
    radius: f64,
) -> Result<(), Failure> {
    let array = Prs3dToolSphere::create(
        radius,
        object.nb_splits_quadric(),
        object.nb_splits_quadric(),
        &Transform3::default(),
    )?;
    light_group(
        prs,
        object.attributes().shading_aspect().aspect().cast(),
        &array.cast(),
        true,
    )
}
pub(super) fn light_cone_frame(
    position: Point3,
    direction: Direction3,
) -> Result<Transform3, Failure> {
    let mut transform = Transform3::default();
    transform.set_transformation_between(
        Axis3::from_axis2(Axis2::new(position, direction)?),
        Axis3::default(),
    );
    Ok(transform)
}
pub(super) fn compute_light_positional<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Presentation,
    mode: i32,
) -> Result<(), Failure> {
    let light = object.light();
    let radius = if object.is_zoomable() && light.has_range() {
        f64::from(light.range())
    } else {
        0.0
    };
    if mode == 0 && radius > 0.0 && object.to_display_range() {
        light_sphere(object, prs, radius)?;
    }
    light_point(
        prs,
        object.attributes().point_aspect().aspect().cast(),
        Point3::default(),
    )
}
pub(super) fn compute_light_spot<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Presentation,
    mode: i32,
) -> Result<(), Failure> {
    let distance = light_distance(object);
    light_point(
        prs,
        object.attributes().point_aspect().aspect().cast(),
        Point3::default(),
    )?;
    spot_lines(object, prs, distance)?;
    if mode == 0 && object.to_display_range() {
        spot_cone(object, prs, distance)?;
    }
    Ok(())
}
fn spot_lines<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Presentation,
    distance: f64,
) -> Result<(), Failure> {
    let lines = Handle::new(Graphic3dArrayOfSegments::new(
        2,
        0,
        GRAPHIC3D_ARRAY_FLAGS_NONE,
    )?);
    lines.add_vertex_pnt(&Point3::default())?;
    lines.add_vertex_pnt(&Point3::new(0.0, 0.0, -distance))?;
    light_group(prs, light_shadow_aspect(object), &lines.cast(), true)?;
    light_group(prs, light_arrow_aspect(object), &lines.cast(), true)
}
fn spot_cone<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Presentation,
    distance: f64,
) -> Result<(), Failure> {
    let array = spot_triangles(object, distance)?;
    light_group(
        prs,
        object.attributes().shading_aspect().aspect().cast(),
        &array.cast(),
        true,
    )
}
pub(super) fn light_arrow_aspect<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Handle<dyn Graphic3dAspectsSource> {
    object.attributes().arrow_aspect().aspect().cast()
}
pub(super) fn light_shadow_aspect<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Handle<dyn Graphic3dAspectsSource> {
    object.light_source_base().visual.read().arrow_shadow.cast()
}
pub(super) fn light_disabled_aspect<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Handle<dyn Graphic3dAspectsSource> {
    object
        .light_source_base()
        .visual
        .read()
        .disabled_marker
        .cast()
}
pub(super) fn light_marker_aspect<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Handle<dyn Graphic3dAspectsSource> {
    object.attributes().point_aspect().aspect().cast()
}
fn light_distance<O: AisLightSourceSource + ?Sized>(object: &O) -> f64 {
    let light = object.light();
    if object.is_zoomable() && light.has_range() {
        return f64::from(light.range());
    }
    object.size()
}
fn spot_triangles<O: AisLightSourceSource + ?Sized>(
    object: &O,
    distance: f64,
) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
    // Native selects std::tan(float), then widens that float result for the radius.
    let radius = distance * f64::from((object.light().angle() / 2.0).tan());
    let transform = light_cone_frame(Point3::new(0.0, 0.0, -distance), Direction3::Z)?;
    Prs3dToolCylinder::create(
        radius,
        0.0,
        distance,
        object.nb_splits_quadric(),
        object.nb_splits_quadric(),
        &transform,
    )
}
