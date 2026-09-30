// Rust translation of the static Project functions of BRepLib_MakeEdge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projection of a vertex on the curve of `BRepLib_MakeEdge`: a curve end within
//! the vertex tolerance, else the nearest Extrema_ExtPC extremum.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::gp::Point3;

/// The native default TolF of `Extrema_ExtPC(P, C)`.
const EXTREMA_TOL_F: f64 = 1.0e-10;

/// `Project(C, V, p)`: the parameter of `vertex` on the 3d curve, or None when no
/// point of the curve lies within the vertex tolerance.
pub(super) fn project_on_curve(
    curve: &Handle<dyn GeomCurve>,
    vertex: &Vertex,
) -> Result<Option<f64>, Failure> {
    let tolerance: f64 = BRepTool::vertex_tolerance(vertex)?;
    let eps2: f64 = tolerance * tolerance;
    let point: Point3 = BRepTool::pnt(vertex)?;
    let adaptor = GeomAdaptorCurve::with_curve(curve)?;
    if let Some(parameter) = end_within(&adaptor, point, eps2)? {
        return Ok(Some(parameter));
    }
    let Some((parameter, _, dist2)) = nearest_extremum(&adaptor, point)? else {
        return Ok(None);
    };
    Ok((dist2 <= eps2).then_some(parameter))
}

/// `Project(C, S, V, p)`: the parameter of `vertex` on the 2d curve of the surface; an
/// extremum counts only within Precision::Confusion() of the vertex point.
pub(super) fn project_on_surface_curve(
    curve: &Handle<dyn Geom2dCurve>,
    surface: &Handle<dyn GeomSurface>,
    vertex: &Vertex,
) -> Result<Option<f64>, Failure> {
    let point: Point3 = BRepTool::pnt(vertex)?;
    let tolerance: f64 = BRepTool::vertex_tolerance(vertex)?;
    let eps2: f64 = tolerance * tolerance;
    let curve2d: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(curve)?).cast();
    let adapted: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(surface)?).cast();
    let on_surface = Adaptor3dCurveOnSurface::with_curve(&curve2d, &adapted)?;
    if let Some(parameter) = end_within(&on_surface, point, eps2)? {
        return Ok(Some(parameter));
    }
    let Some((parameter, found, _)) = nearest_extremum(&on_surface, point)? else {
        return Ok(None);
    };
    let near: bool = point.square_distance(&found) <= precision::square_confusion();
    Ok(near.then_some(parameter))
}

/// The end check of both projections: the nearer curve end when it lies within
/// `eps2` (square distance) of the point; equal distances never match.
fn end_within(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
    eps2: f64,
) -> Result<Option<f64>, Failure> {
    let (first, last): (f64, f64) = (curve.first_parameter()?, curve.last_parameter()?);
    let d1: f64 = curve.value(first)?.square_distance(&point);
    let d2: f64 = curve.value(last)?.square_distance(&point);
    if d1 < d2 && d1 <= eps2 {
        return Ok(Some(first));
    }
    if d2 < d1 && d2 <= eps2 {
        return Ok(Some(last));
    }
    Ok(None)
}

/// The extremum of smallest square distance (the first of equal ones): its parameter,
/// point and square distance; None when Extrema is not done or finds nothing.
fn nearest_extremum(
    curve: &dyn Adaptor3dCurve,
    point: Point3,
) -> Result<Option<(f64, Point3, f64)>, Failure> {
    let extrema = ExtPC::with_curve(point, curve, EXTREMA_TOL_F)?;
    if !extrema.is_done() {
        return Ok(None);
    }
    let mut nearest: Option<(f64, Point3, f64)> = None;
    let mut dist2: f64 = f64::MAX;
    for index in 1..=extrema.nb_ext()? {
        let dist2min: f64 = extrema.square_distance(index)?;
        if dist2min < dist2 {
            let found = extrema.point(index)?;
            nearest = Some((found.parameter(), *found.value(), dist2min));
            dist2 = dist2min;
        }
    }
    Ok(nearest)
}
