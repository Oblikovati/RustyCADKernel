//! OCCT snapshots arrow dimensions before callbacks and emits corner shafts last.
use crate::prs3d::{Prs3dArrow, Prs3dPlaneAspect};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use rustycad_tkservice::graphic3d::*;
struct PlaneArrowGeometry {
    normal: Direction3,
    translation: Vector3,
    angle: f64,
    length: f64,
}
fn arrow_geometry(plane: &GeomPlane, aspect: &Prs3dPlaneAspect) -> PlaneArrowGeometry {
    let normal = plane.pln().axis().direction();
    PlaneArrowGeometry {
        normal,
        translation: Vector3::from(normal).scaled(aspect.arrows_size()),
        length: aspect.arrows_length(),
        angle: aspect.arrows_angle(),
    }
}
pub(super) fn draw_plane_arrows(
    prs: &Handle<dyn Graphic3dStructureSource>,
    group: &Handle<dyn Graphic3dGroup>,
    plane: &GeomPlane,
    aspect: &Handle<Prs3dPlaneAspect>,
    extents: [f64; 2],
) -> Result<(), Failure> {
    let geometry = arrow_geometry(plane, aspect);
    group.set_primitives_aspect(&aspect.arrow_aspect().aspect().cast());
    if aspect.display_center_arrow() {
        let vertices = Handle::new(Graphic3dArrayOfSegments::new(2, 0, 0)?);
        let tip = append_shaft(&vertices, &geometry, plane.pln().location())?;
        group.add_primitive_array(&vertices.cast(), true);
        draw_arrowhead(prs, &geometry, tip)?;
    }
    if aspect.display_edges_arrows() {
        draw_corner_arrows(prs, group, plane, &geometry, extents)?;
    }
    Ok(())
}
fn draw_corner_arrows(
    prs: &Handle<dyn Graphic3dStructureSource>,
    group: &Handle<dyn Graphic3dGroup>,
    plane: &GeomPlane,
    geometry: &PlaneArrowGeometry,
    extents: [f64; 2],
) -> Result<(), Failure> {
    let [x, y] = extents;
    let vertices = Handle::new(Graphic3dArrayOfSegments::new(8, 0, 0)?);
    for (u, v) in [(-x, -y), (-x, y), (x, y), (x, -y)] {
        let tip = append_shaft(&vertices, geometry, plane.value(u, v)?)?;
        draw_arrowhead(prs, geometry, tip)?;
    }
    group.add_primitive_array(&vertices.cast(), true);
    Ok(())
}
fn append_shaft(
    vertices: &Handle<Graphic3dArrayOfSegments>,
    geometry: &PlaneArrowGeometry,
    origin: Point3,
) -> Result<Point3, Failure> {
    let tip = origin.translated(geometry.translation);
    vertices.add_vertex_pnt(&origin)?;
    vertices.add_vertex_pnt(&tip)?;
    Ok(tip)
}
fn draw_arrowhead(
    prs: &Handle<dyn Graphic3dStructureSource>,
    geometry: &PlaneArrowGeometry,
    tip: Point3,
) -> Result<(), Failure> {
    Prs3dArrow::draw(
        &prs.current_group()?,
        &tip,
        &geometry.normal,
        geometry.angle,
        geometry.length,
    )
}
