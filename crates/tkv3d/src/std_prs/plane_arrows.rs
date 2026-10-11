//! OCCT emits edge arrowheads before the combined shaft array; retain that order.
use crate::prs3d::{Prs3dArrow, Prs3dPlaneAspect};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkservice::graphic3d::*;

pub(super) fn draw_plane_arrows(
    prs: &Handle<dyn Graphic3dStructureSource>,
    group: &Handle<dyn Graphic3dGroup>,
    plane: &GeomPlane,
    aspect: &Handle<Prs3dPlaneAspect>,
) -> Result<(), Failure> {
    group.set_primitives_aspect(&aspect.arrow_aspect().aspect().cast());
    if aspect.display_center_arrow() {
        let vertices = Handle::new(Graphic3dArrayOfSegments::new(2, 0, 0)?);
        let origin = plane.pln().location();
        let tip = append_shaft(&vertices, plane, aspect, origin)?;
        group.add_primitive_array(&vertices.cast(), true);
        draw_arrowhead(prs, plane, aspect, tip)?;
    }
    if aspect.display_edges_arrows() {
        draw_corner_arrows(prs, group, plane, aspect)?;
    }
    Ok(())
}
fn draw_corner_arrows(
    prs: &Handle<dyn Graphic3dStructureSource>,
    group: &Handle<dyn Graphic3dGroup>,
    plane: &GeomPlane,
    aspect: &Handle<Prs3dPlaneAspect>,
) -> Result<(), Failure> {
    let x = 0.5 * aspect.plane_x_length();
    let y = 0.5 * aspect.plane_y_length();
    let vertices = Handle::new(Graphic3dArrayOfSegments::new(8, 0, 0)?);
    for (u, v) in [(-x, -y), (-x, y), (x, y), (x, -y)] {
        let tip = append_shaft(&vertices, plane, aspect, plane.value(u, v)?)?;
        draw_arrowhead(prs, plane, aspect, tip)?;
    }
    group.add_primitive_array(&vertices.cast(), true);
    Ok(())
}
fn append_shaft(
    vertices: &Handle<Graphic3dArrayOfSegments>,
    plane: &GeomPlane,
    aspect: &Handle<Prs3dPlaneAspect>,
    origin: Point3,
) -> Result<Point3, Failure> {
    let translation = Vector3::from(plane.pln().axis().direction()).scaled(aspect.arrows_size());
    let tip = origin.translated(translation);
    vertices.add_vertex_pnt(&origin)?;
    vertices.add_vertex_pnt(&tip)?;
    Ok(tip)
}
fn draw_arrowhead(
    prs: &Handle<dyn Graphic3dStructureSource>,
    plane: &GeomPlane,
    aspect: &Handle<Prs3dPlaneAspect>,
    tip: Point3,
) -> Result<(), Failure> {
    Prs3dArrow::draw(
        &prs.current_group()?,
        &tip,
        &plane.pln().axis().direction(),
        aspect.arrows_angle(),
        aspect.arrows_length(),
    )
}
