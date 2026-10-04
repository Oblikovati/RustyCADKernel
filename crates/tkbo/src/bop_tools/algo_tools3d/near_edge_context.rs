use super::{point_in_face_from_edge, point_near_edge_step};
use crate::int_tools::Context;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom_adaptor::GeomAdaptorSurface};
use rustycad_tkmath::{
    geom_abs::SurfaceType,
    gp::{Point2, Point3},
};
/// Classify the nearby point, falling back to a bounded hatch domain; `point_near_edge_with_step(&edge,&face,0.5,1e-4,&mut uv,&mut point,&context)?`.
pub fn point_near_edge_with_step(
    edge: &Edge,
    face: &Face,
    parameter: f64,
    step: f64,
    uv: &mut Point2,
    point: &mut Point3,
    context: &Context,
) -> Result<i32, Failure> {
    let status = point_near_edge_step(edge, face, parameter, step, uv, point)?;
    if status == 1 || context.is_point_in_on_face(face, *uv)? {
        return Ok(status);
    }
    let mut candidate = Point3::default();
    let mut candidate_uv = Point2::default();
    if point_in_face_from_edge(
        face,
        edge,
        parameter,
        step,
        &mut candidate,
        &mut candidate_uv,
        context,
    )? != 0
    {
        return Ok(2);
    }
    *point = candidate;
    *uv = candidate_uv;
    Ok(0)
}
/// Choose a tolerance-dependent step and classify the result; `point_near_edge_at(&edge,&face,0.5,&mut uv,&mut point,&context)?`.
pub fn point_near_edge_at(
    edge: &Edge,
    face: &Face,
    parameter: f64,
    uv: &mut Point2,
    point: &mut Point3,
    context: &Context,
) -> Result<i32, Failure> {
    let surface = BRepTool::surface(face)?;
    let kind = GeomAdaptorSurface::with_surface(&surface)?.get_type()?;
    let mut step = 10. * super::min_step_in_2d();
    if matches!(kind, SurfaceType::Cylinder | SurfaceType::Sphere) {
        step *= 10.;
    }
    let tolerance = 2. * (BRepTool::edge_tolerance(edge)? + BRepTool::face_tolerance(face)?);
    if tolerance > step {
        step = tolerance;
    }
    point_near_edge_with_step(edge, face, parameter, step, uv, point, context)
}
