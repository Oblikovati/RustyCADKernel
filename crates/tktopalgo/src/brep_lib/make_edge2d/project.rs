// Rust translation of the static Point and Project functions of BRepLib_MakeEdge2d.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The passage between the current plane and its parameter space for
//! `BRepLib_MakeEdge2d`, and the projection of a vertex on a 2d curve.
use crate::brep_lib::errors::null_object;
use crate::brep_lib::globals;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkgeombase::extrema::ExtPC2d;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Point3};

/// The native default TolF of `Extrema_ExtPC2d(P, C)`.
const EXTREMA_TOL_F: f64 = 1.0e-10;

/// The Geom plane of the current `BRepLib::Plane()` handle (never null: the read
/// creates the default plane).
fn plane_surface(plane: &Handle<GeomPlane>) -> Result<&GeomPlane, Failure> {
    plane
        .get()
        .ok_or_else(|| null_object("BRepLib::Plane(): null plane; expected Geom_Plane(gp::XOY())"))
}

/// `Point(P)`: the point of the current plane at the parameters `P`.
pub(super) fn plane_point(point: Point2) -> Result<Point3, Failure> {
    let plane: Handle<GeomPlane> = globals::plane();
    plane_surface(&plane)?.value(point.x(), point.y())
}

/// `Project(Ve)`: the parameters of the vertex point on the current plane.
pub(super) fn plane_parameters(vertex: &Vertex) -> Result<Point2, Failure> {
    let point: Point3 = BRepTool::pnt(vertex)?;
    let plane: Handle<GeomPlane> = globals::plane();
    let pln = plane_surface(&plane)?.pln();
    let (u, v) = elslib::plane_parameters(pln.position(), point);
    Ok(Point2::new(u, v))
}

/// `Project(C, V, p)`: the parameter of the vertex parameters on the 2d curve: the
/// closed form for a line or a circle, else the nearest Extrema_ExtPC2d extremum;
/// None when Extrema is not done. Natively a done Extrema without extremum leaves the
/// caller's parameter uninitialized; the port returns 0.0 then.
pub(super) fn project_on_curve2d(
    curve: &Handle<dyn Geom2dCurve>,
    vertex: &Vertex,
) -> Result<Option<f64>, Failure> {
    let point: Point2 = plane_parameters(vertex)?;
    let adaptor = Geom2dAdaptorCurve::with_curve(curve)?;
    match adaptor.get_type()? {
        CurveType::Line => Ok(Some(elclib::line_parameter_2d(
            adaptor.line()?.position(),
            point,
        ))),
        CurveType::Circle => Ok(Some(elclib::circle_parameter_2d(
            adaptor.circle()?.position(),
            point,
        )?)),
        _ => nearest_parameter(&adaptor, point),
    }
}

/// The parameter of the extremum of smallest square distance (the first of equal
/// ones), all extrema counted (the IsMin filter is commented out natively, OCC16852).
fn nearest_parameter(
    adaptor: &dyn Adaptor2dCurve2d,
    point: Point2,
) -> Result<Option<f64>, Failure> {
    let extrema = ExtPC2d::with_curve(point, adaptor, EXTREMA_TOL_F)?;
    if !extrema.is_done() {
        return Ok(None);
    }
    let mut parameter: f64 = 0.0;
    let mut d2: f64 = f64::MAX;
    for index in 1..=extrema.nb_ext()? {
        let dd2: f64 = extrema.square_distance(index)?;
        if dd2 < d2 {
            d2 = dd2;
            parameter = extrema.point(index)?.parameter();
        }
    }
    Ok(Some(parameter))
}
