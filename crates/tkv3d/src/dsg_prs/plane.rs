//! Martensite plane builders preserve OCCT 3d097a0 topology and aspect ordering.
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;

/// Closed datum boundary; `DsgPrsXYZPlanePresentation::add(&prs, &drawer, &a, &b, &c)?`.
pub struct DsgPrsXYZPlanePresentation;
impl DsgPrsXYZPlanePresentation {
    /// Submit the native four-vertex polyline; `Self::add(&prs, &drawer, &a, &b, &c)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        drawer: &Handle<Prs3dDrawer>,
        first: &Point3,
        second: &Point3,
        third: &Point3,
    ) -> Result<(), Failure> {
        let group = plane_group(prs, drawer)?;
        let vertices = Handle::new(Graphic3dArrayOfPolylines::new(4, 0, 0, 0)?);
        append_plane_loop(&vertices.cast(), first, second, third)?;
        group.add_primitive_array(&vertices.cast(), true);
        Ok(())
    }
}
/// Filled datum triangle retaining native polygon input; `DsgPrsShadedPlanePresentation::add(&prs, &drawer, &a, &b, &c)?`.
pub struct DsgPrsShadedPlanePresentation;
impl DsgPrsShadedPlanePresentation {
    /// Switch from edges to shading before submission; `Self::add(&prs, &drawer, &a, &b, &c)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        drawer: &Handle<Prs3dDrawer>,
        first: &Point3,
        second: &Point3,
        third: &Point3,
    ) -> Result<(), Failure> {
        let group = plane_group(prs, drawer)?;
        group.set_primitives_aspect(&drawer.shading_aspect().aspect().cast());
        let vertices = Handle::new(Graphic3dArrayOfPolygons::new(4, 0, 0, 0)?);
        append_plane_loop(&vertices.cast(), first, second, third)?;
        group.add_primitive_array(&vertices.cast(), true);
        Ok(())
    }
}
fn plane_group(
    prs: &Handle<dyn Graphic3dStructureSource>,
    drawer: &Handle<Prs3dDrawer>,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    let group = prs.current_group()?;
    group.set_primitives_aspect(&drawer.plane_aspect().edges_aspect().aspect().cast());
    Ok(group)
}
fn append_plane_loop(
    vertices: &Handle<Graphic3dArrayOfPrimitives>,
    first: &Point3,
    second: &Point3,
    third: &Point3,
) -> Result<(), Failure> {
    for point in [first, second, third, first] {
        vertices.add_vertex_pnt(point)?;
    }
    Ok(())
}
