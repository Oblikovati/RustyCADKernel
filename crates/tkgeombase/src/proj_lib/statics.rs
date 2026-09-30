// Rust translation of ModelingData/TKGeomBase/ProjLib/ProjLib.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static members of the `ProjLib` class: the `Project` overloads of points,
//! lines and conics onto the elementary surfaces, `MakePCurveOfType` and
//! `IsAnaSurf`. A curve overload builds the elementary projector and returns its
//! typed result, so it raises Standard_NoSuchObject when the projector does not
//! give that kind (a line not parallel to a cylinder axis, for example).
use super::{Cone, Cylinder, Plane, ProjectedCurve, Sphere, Torus};
use crate::native_failure::failure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dEllipse, Geom2dHyperbola, Geom2dLine, Geom2dParabola,
};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{
    Circle2, Circle3, Cone3, Cylinder3, Ellipse2, Ellipse3, Hyperbola2, Hyperbola3, Line2, Line3,
    Parabola2, Parabola3, Plane3, Point2, Point3, Sphere3, Torus3,
};

// `ElSLib::Parameters` as a 2D point.
fn uv(parameters: (f64, f64)) -> Point2 {
    Point2::new(parameters.0, parameters.1)
}

/// `ProjLib::Project(gp_Pln, gp_Pnt)`: the plane parameters of a point;
/// `project_point_on_plane(&plane, point)`.
pub fn project_point_on_plane(plane: &Plane3, point: Point3) -> Point2 {
    uv(elslib::parameters(plane, point))
}

/// `ProjLib::Project(gp_Pln, gp_Lin)`; `project_line_on_plane(&plane, &line)?`.
pub fn project_line_on_plane(plane: &Plane3, line: &Line3) -> Result<Line2, Failure> {
    Plane::with_line(plane, line)?.line()
}

/// `ProjLib::Project(gp_Pln, gp_Circ)`; `project_circle_on_plane(&plane, &circle)?`.
pub fn project_circle_on_plane(plane: &Plane3, circle: &Circle3) -> Result<Circle2, Failure> {
    Plane::with_circle(plane, circle)?.circle()
}

/// `ProjLib::Project(gp_Pln, gp_Elips)`; `project_ellipse_on_plane(&plane, &ellipse)?`.
pub fn project_ellipse_on_plane(plane: &Plane3, ellipse: &Ellipse3) -> Result<Ellipse2, Failure> {
    Plane::with_ellipse(plane, ellipse)?.ellipse()
}

/// `ProjLib::Project(gp_Pln, gp_Parab)`; `project_parabola_on_plane(&plane, &parabola)?`.
pub fn project_parabola_on_plane(
    plane: &Plane3,
    parabola: &Parabola3,
) -> Result<Parabola2, Failure> {
    Plane::with_parabola(plane, parabola)?.parabola()
}

/// `ProjLib::Project(gp_Pln, gp_Hypr)`; `project_hyperbola_on_plane(&plane, &hyperbola)?`.
pub fn project_hyperbola_on_plane(
    plane: &Plane3,
    hyperbola: &Hyperbola3,
) -> Result<Hyperbola2, Failure> {
    Plane::with_hyperbola(plane, hyperbola)?.hyperbola()
}

/// `ProjLib::Project(gp_Cylinder, gp_Pnt)`; `project_point_on_cylinder(&cylinder, point)`.
pub fn project_point_on_cylinder(cylinder: &Cylinder3, point: Point3) -> Point2 {
    uv(elslib::parameters(cylinder, point))
}

/// `ProjLib::Project(gp_Cylinder, gp_Lin)`; `project_line_on_cylinder(&cylinder, &line)?`.
pub fn project_line_on_cylinder(cylinder: &Cylinder3, line: &Line3) -> Result<Line2, Failure> {
    Cylinder::with_line(cylinder, line).line()
}

/// `ProjLib::Project(gp_Cylinder, gp_Circ)`; `project_circle_on_cylinder(&cylinder, &circle)?`.
pub fn project_circle_on_cylinder(
    cylinder: &Cylinder3,
    circle: &Circle3,
) -> Result<Line2, Failure> {
    Cylinder::with_circle(cylinder, circle)?.line()
}

/// `ProjLib::Project(gp_Cone, gp_Pnt)`; `project_point_on_cone(&cone, point)`.
pub fn project_point_on_cone(cone: &Cone3, point: Point3) -> Point2 {
    uv(elslib::parameters(cone, point))
}

/// `ProjLib::Project(gp_Cone, gp_Lin)`; `project_line_on_cone(&cone, &line)?`.
pub fn project_line_on_cone(cone: &Cone3, line: &Line3) -> Result<Line2, Failure> {
    Cone::with_line(cone, line)?.line()
}

/// `ProjLib::Project(gp_Cone, gp_Circ)`: the projector sets the Line type before
/// its axis test, so a circle not normal to the axis returns the default line
/// instead of raising; `project_circle_on_cone(&cone, &circle)?`.
pub fn project_circle_on_cone(cone: &Cone3, circle: &Circle3) -> Result<Line2, Failure> {
    Cone::with_circle(cone, circle)?.line()
}

/// `ProjLib::Project(gp_Sphere, gp_Pnt)`; `project_point_on_sphere(&sphere, point)`.
pub fn project_point_on_sphere(sphere: &Sphere3, point: Point3) -> Point2 {
    uv(elslib::parameters(sphere, point))
}

/// `ProjLib::Project(gp_Sphere, gp_Circ)`; `project_circle_on_sphere(&sphere, &circle)?`.
pub fn project_circle_on_sphere(sphere: &Sphere3, circle: &Circle3) -> Result<Line2, Failure> {
    Sphere::with_circle(sphere, circle)?.line()
}

/// `ProjLib::Project(gp_Torus, gp_Pnt)`; `project_point_on_torus(&torus, point)`.
pub fn project_point_on_torus(torus: &Torus3, point: Point3) -> Point2 {
    uv(elslib::parameters(torus, point))
}

/// `ProjLib::Project(gp_Torus, gp_Circ)`; `project_circle_on_torus(&torus, &circle)?`.
pub fn project_circle_on_torus(torus: &Torus3, circle: &Circle3) -> Result<Line2, Failure> {
    Torus::with_circle(torus, circle)?.line()
}

/// `ProjLib::MakePCurveOfType(PC, C2D)`: a new Geom2d curve of the projected kind
/// (the B-spline handle itself for a B-spline); Standard_NotImplemented for a
/// Bezier or another kind; `make_pcurve_of_type(&projected)?`.
pub fn make_pcurve_of_type(projected: &ProjectedCurve) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    match projected.get_type()? {
        CurveType::Line => Ok(Handle::new(Geom2dLine::from_lin2d(projected.line()?)).cast()),
        CurveType::Circle => Ok(Handle::new(Geom2dCircle::from_circ2d(projected.circle()?)).cast()),
        CurveType::Ellipse => {
            Ok(Handle::new(Geom2dEllipse::from_elips2d(projected.ellipse()?)).cast())
        }
        CurveType::Parabola => {
            Ok(Handle::new(Geom2dParabola::from_parab2d(projected.parabola()?)).cast())
        }
        CurveType::Hyperbola => {
            Ok(Handle::new(Geom2dHyperbola::from_hypr2d(projected.hyperbola()?)).cast())
        }
        CurveType::BSplineCurve => Ok(projected.bspline()?.cast()),
        _ => Err(failure(
            "Standard_NotImplemented",
            "ProjLib::MakePCurveOfType",
        )),
    }
}

/// `ProjLib::IsAnaSurf(theAS)`: whether the surface is a plane, cylinder, cone,
/// sphere or torus; `is_ana_surf(&surface)?`.
pub fn is_ana_surf(surface: &Handle<dyn Adaptor3dSurface>) -> Result<bool, Failure> {
    let adaptor = surface.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "ProjLib::IsAnaSurf: the surface handle is null",
        )
    })?;
    Ok(matches!(
        adaptor.get_type()?,
        SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
    ))
}
