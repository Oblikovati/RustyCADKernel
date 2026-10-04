//! Native line/cylinder and circle/plane correction passes.
use super::EdgeFace;
use super::touch::{check_touch, check_touch_vertex};
use crate::int_tools::Tools;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};

pub(super) fn postprocess_touch_parts(edge_face: &mut EdgeFace) -> Result<(), Failure> {
    if !needs_touch_postprocess(edge_face)? || is_circle_plane_exempt(edge_face)? {
        return Ok(());
    }
    for index in 1..=edge_face.common_parts.len() {
        postprocess_part(edge_face, index)?;
    }
    Ok(())
}

fn needs_touch_postprocess(edge_face: &EdgeFace) -> Result<bool, Failure> {
    let curve = edge_face.curve.get().expect("initialized edge-face curve");
    let surface = edge_face
        .surface
        .get()
        .expect("initialized edge-face surface");
    let curve_type = curve.get_type()?;
    let surface_type = surface.get_type()?;
    Ok(
        curve_type == CurveType::Line && surface_type == SurfaceType::Cylinder
            || curve_type == CurveType::Circle && surface_type == SurfaceType::Plane,
    )
}

fn is_circle_plane_exempt(edge_face: &EdgeFace) -> Result<bool, Failure> {
    let curve = edge_face.curve.get().expect("initialized edge-face curve");
    if curve.get_type()? != CurveType::Circle {
        return Ok(false);
    }
    let surface = edge_face
        .surface
        .get()
        .expect("initialized edge-face surface");
    if surface.get_type()? != SurfaceType::Plane {
        return Ok(false);
    }
    let circle = curve.circle()?;
    let plane = surface.plane()?;
    let coplanar =
        Tools::is_dirs_coinside(circle.position().direction(), plane.position().direction());
    let radius = (plane.distance(circle.location()) - circle.radius()).abs() < edge_face.criteria;
    Ok(coplanar || radius)
}

fn postprocess_part(edge_face: &mut EdgeFace, index: usize) -> Result<(), Failure> {
    let part = edge_face.common_parts.value(index)?;
    let shape_type = part.shape_type();
    let (first, last) = part.range1_bounds();
    match shape_type {
        ShapeEnum::Edge => upgrade_touch_edge(edge_face, index, first, last),
        ShapeEnum::Vertex => refine_touch_vertex(edge_face, index, first, last),
        _ => Ok(()),
    }
}

fn upgrade_touch_edge(
    edge_face: &mut EdgeFace,
    index: usize,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let Some(parameter) = check_touch(edge_face, first, last)? else {
        return Ok(());
    };
    let part = edge_face.common_parts.change_value(index)?;
    part.set_type(ShapeEnum::Vertex);
    part.set_vertex_parameter1(parameter);
    Ok(())
}

fn refine_touch_vertex(
    edge_face: &mut EdgeFace,
    index: usize,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let Some(parameter) = check_touch_vertex(edge_face, first, last)? else {
        return Ok(());
    };
    edge_face
        .common_parts
        .change_value(index)?
        .set_vertex_parameter1(parameter);
    Ok(())
}
