//! Hatching-based PointInFace overloads from pinned BOPTools_AlgoTools3D.cxx.
use crate::int_tools::{Context, Tools};
use rustycad_tkbrep::{brep::BRepTool, topods::Face};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::{
    geom2d::{Geom2dCurve, Geom2dLine},
    geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkgeomalgo::{geom2d_hatch::Hatcher, hatch_gen::Domain};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Point3};
/// Find the first bounded hatch domain; `point_in_face_curve(&face,&line,&mut point,&mut uv,&context,0.)?`.
pub fn point_in_face_curve(
    face: &Face,
    curve: &Handle<dyn Geom2dCurve>,
    point: &mut Point3,
    uv: &mut Point2,
    context: &Context,
    step: f64,
) -> Result<i32, Failure> {
    let cached = context.hatcher(face)?;
    let mut hatcher = cached.write();
    let adaptor = Geom2dAdaptorCurve::with_curve(curve)?;
    hatcher.clr_hatchings();
    let index = hatcher.add_hatching(adaptor);
    let status = sample_hatching(&mut hatcher, index, face, curve, point, uv, step)?;
    hatcher.rem_hatching(index)?;
    Ok(status)
}
#[allow(clippy::too_many_arguments)]
fn sample_hatching(
    hatcher: &mut Hatcher,
    index: i32,
    face: &Face,
    curve: &Handle<dyn Geom2dCurve>,
    point: &mut Point3,
    uv: &mut Point2,
    step: f64,
) -> Result<i32, Failure> {
    hatcher.trim()?;
    if !hatcher.trim_done(index)? {
        return Ok(1);
    }
    hatcher.compute_domains_of(index)?;
    if !hatcher.is_done(index)? || hatcher.nb_domains(index)? == 0 {
        return Ok(2);
    }
    sample_domain(hatcher.domain(index, 1)?, face, curve, point, uv, step)
}
fn sample_domain(
    domain: &Domain,
    face: &Face,
    curve: &Handle<dyn Geom2dCurve>,
    point: &mut Point3,
    uv: &mut Point2,
    step: f64,
) -> Result<i32, Failure> {
    if !domain.has_first_point() {
        return Ok(3);
    }
    if !domain.has_second_point() {
        return Ok(4);
    }
    let parameter = domain_parameter(domain, step)?;
    let surface = BRepTool::surface(face)?;
    *uv = curve.value(parameter)?;
    *point = surface.value(uv.x(), uv.y())?;
    Ok(0)
}
fn domain_parameter(domain: &Domain, step: f64) -> Result<f64, Failure> {
    let first = domain.first_point()?.parameter();
    let last = domain.second_point()?.parameter();
    Ok(if step > 0. && last - first > step {
        first + step
    } else {
        Tools::intermediate_point(first, last)
    })
}
/// Try two vertical hatch lines using the cached face bounds; `point_in_face(&face,&mut point,&mut uv,&context)?`.
pub fn point_in_face(
    face: &Face,
    point: &mut Point3,
    uv: &mut Point2,
    context: &Context,
) -> Result<i32, Failure> {
    let (first, last, _, _) = context.uv_bounds(face)?;
    let mut coordinate = Tools::intermediate_point(first, last);
    let mut status = 1;
    for _ in 0..2 {
        let curve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
            Point2::new(coordinate, 0.),
            Direction2::Y,
        )))
        .cast();
        status = point_in_face_curve(face, &curve, point, uv, context, 0.)?;
        if status == 0 {
            break;
        }
        coordinate = last - (coordinate - first);
    }
    Ok(status)
}

/// Hatch an inward ray from a stored edge pcurve; `point_in_face_from_edge(&face,&edge,0.5,1e-4,&mut point,&mut uv,&context)?`.
pub fn point_in_face_from_edge(
    face: &Face,
    edge: &rustycad_tkbrep::topods::Edge,
    parameter: f64,
    step: f64,
    point: &mut Point3,
    uv: &mut Point2,
    context: &Context,
) -> Result<i32, Failure> {
    let curve = BRepTool::curve_on_face(edge, face)?;
    if curve.pcurve.is_null() {
        return Ok(5);
    }
    let (origin, tangent) = curve.pcurve.d1(parameter)?;
    let direction =
        super::near_edge::interior_direction(tangent, edge.orientation(), face.orientation())?;
    let ray: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::new(origin, direction))).cast();
    let ray: Handle<dyn Geom2dCurve> =
        Handle::new(rustycad_tkg2d::geom2d::Geom2dTrimmedCurve::new(
            &ray,
            0.,
            rustycad_tkernel::precision::infinite(),
            true,
            true,
        )?)
        .cast();
    point_in_face_curve(face, &ray, point, uv, context, step)
}
