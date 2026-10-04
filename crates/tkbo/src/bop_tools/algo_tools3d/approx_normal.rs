use super::{
    get_normal_to_surface, point_near_edge_at, point_near_edge_step, point_near_edge_with_step,
};
use crate::int_tools::Context;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction3, Point2, Point3};
/// Sample beside an edge using an explicit UV step; `approx_normal_with_step(&edge,&face,0.5,&mut point,&mut normal,1e-5)?`.
pub fn approx_normal_with_step(
    edge: &Edge,
    face: &Face,
    parameter: f64,
    point: &mut Point3,
    normal: &mut Direction3,
    step: f64,
) -> Result<bool, Failure> {
    let mut uv = Point2::default();
    let status = point_near_edge_step(edge, face, parameter, step, &mut uv, point)?;
    finish_normal(face, uv, status, normal)
}
/// Use the context's tolerance-derived nearby point; `approx_normal_on_edge(&edge,&face,0.5,&mut point,&mut normal,&context)?`.
pub fn approx_normal_on_edge(
    edge: &Edge,
    face: &Face,
    parameter: f64,
    point: &mut Point3,
    normal: &mut Direction3,
    context: &Context,
) -> Result<bool, Failure> {
    let mut uv = Point2::default();
    let status = point_near_edge_at(edge, face, parameter, &mut uv, point, context)?;
    finish_normal(face, uv, status, normal)
}
/// Use an explicit step with hatching fallback; `approx_normal_with_context_step(&edge,&face,0.5,1e-5,&mut point,&mut normal,&context)?`.
pub fn approx_normal_with_context_step(
    edge: &Edge,
    face: &Face,
    parameter: f64,
    step: f64,
    point: &mut Point3,
    normal: &mut Direction3,
    context: &Context,
) -> Result<bool, Failure> {
    let mut uv = Point2::default();
    let status = point_near_edge_with_step(edge, face, parameter, step, &mut uv, point, context)?;
    finish_normal(face, uv, status, normal)
}
fn finish_normal(
    face: &Face,
    uv: Point2,
    status: i32,
    normal: &mut Direction3,
) -> Result<bool, Failure> {
    if status == 1 {
        return Ok(false);
    }
    let surface = BRepTool::surface(face)?;
    // Native ignores the normal computation's Boolean and bases success only on the nearby point.
    get_normal_to_surface(&surface, uv.x(), uv.y(), normal)?;
    if face.orientation() == Orientation::Reversed {
        normal.reverse();
    }
    Ok(status == 0)
}
