// Rust translation of the static ComputeCurve3d of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Offset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 3d curve of an edge built from its p-curve: the exact iso circle or line of
//! an analytic surface, the 3d embedding of a p-curve on a plane.
use super::update_edge_curve;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::geom_api;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis1, Axis3, Circle3, Direction2, Line3, Vector3};
use rustycad_tkmath::toploc::Location;
use std::f64::consts::FRAC_PI_2;

/// The axis of the frame direction `XDirection ^ second` at its location.
fn rev_axis(axis: &Axis3, second: rustycad_tkmath::gp::Direction3) -> Result<Axis1, Failure> {
    Ok(Axis1::new(
        axis.location(),
        axis.x_direction().crossed(second)?,
    ))
}

/// The circle of an iso, reversed when the p-curve runs against the axis.
fn circle_curve(ci: Circle3, opposite: bool) -> Result<Handle<dyn GeomCurve>, Failure> {
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(ci)).cast();
    if opposite {
        circle.get().ok_or_else(null_curve)?.reverse()?;
    }
    Ok(circle)
}

/// The line of an iso, reversed when the p-curve runs against the axis.
fn line_curve(l: Line3, opposite: bool) -> Result<Handle<dyn GeomCurve>, Failure> {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(l)).cast();
    if opposite {
        line.get().ok_or_else(null_curve)?.reverse()?;
    }
    Ok(line)
}

fn null_curve() -> Failure {
    Failure::with_message("Standard_NullObject", "BRepOffset_Offset: null iso curve")
}

/// `ComputeCurve3d(Edge, Curve, Surf, Loc, Tol)`: for a line p-curve along U or V on a
/// sphere, cylinder, cone or torus the exact iso (the edge made degenerated at a pole of
/// the sphere), for any p-curve on a plane its 3d embedding; nothing otherwise.
pub(super) fn compute_curve3d(
    edge: &Edge,
    curve: &Handle<dyn Geom2dCurve>,
    surf: &Handle<dyn GeomSurface>,
    loc: &Location,
    tol: f64,
) -> Result<(), Failure> {
    let c = Geom2dAdaptorCurve::with_curve(curve)?;
    let s = GeomAdaptorSurface::with_surface(surf)?;
    let cty = c.get_type()?;
    let sty = s.get_type()?;
    if sty == SurfaceType::Plane {
        let c3d = geom_api::to_3d(curve, &s.plane()?)?;
        return update_edge_curve(edge, &c3d, loc, tol);
    }
    if cty != CurveType::Line {
        return Ok(());
    }
    let line = c.line()?;
    let d = line.direction();
    let p = line.location();
    let ang = precision::angular();
    if d.is_parallel(Direction2::X, ang) {
        // Iso V.
        let opposite = d.is_opposite(Direction2::X, ang);
        let iso = |ci: Circle3, axis: &Axis3| -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
            let mut ci = ci;
            ci.rotate(rev_axis(axis, axis.y_direction())?, p.x())?;
            Ok(Some(circle_curve(ci, opposite)?))
        };
        let circle = match sty {
            SurfaceType::Sphere => {
                if (p.y().abs() - FRAC_PI_2).abs() < precision::p_confusion() {
                    BRepBuilder::default().degenerated(edge, true)?;
                    return Ok(());
                }
                let sph = s.sphere()?;
                let axis = sph.position();
                iso(elslib::sphere_v_iso(axis, sph.radius(), p.y()), &axis)?
            }
            SurfaceType::Cylinder => {
                let cyl = s.cylinder()?;
                let axis = cyl.position();
                iso(elslib::cylinder_v_iso(axis, cyl.radius(), p.y()), &axis)?
            }
            SurfaceType::Cone => {
                let cone = s.cone()?;
                let axis = cone.position();
                iso(
                    elslib::cone_v_iso(axis, cone.ref_radius(), cone.semi_angle(), p.y()),
                    &axis,
                )?
            }
            SurfaceType::Torus => {
                let tore = s.torus()?;
                let axis = tore.position();
                iso(
                    elslib::torus_v_iso(axis, tore.major_radius(), tore.minor_radius(), p.y()),
                    &axis,
                )?
            }
            _ => None,
        };
        if let Some(curve3d) = circle {
            update_edge_curve(edge, &curve3d, loc, tol)?;
        }
    } else if d.is_parallel(Direction2::Y, ang) {
        // Iso U.
        let opposite = d.is_opposite(Direction2::Y, ang);
        let curve3d = match sty {
            SurfaceType::Sphere => {
                let sph = s.sphere()?;
                let axis = sph.position();
                // The iso 0, rotated to the same parameter (offset of Y), then to the iso U.
                let mut ci = elslib::sphere_u_iso(axis, sph.radius(), 0.);
                ci.rotate(rev_axis(&axis, axis.direction())?, p.y())?;
                ci.rotate(rev_axis(&axis, axis.y_direction())?, p.x())?;
                Some(circle_curve(ci, opposite)?)
            }
            SurfaceType::Cylinder => {
                let cyl = s.cylinder()?;
                let mut l = elslib::cylinder_u_iso(cyl.position(), cyl.radius(), p.x());
                l.translate(Vector3::from(l.direction()) * p.y());
                Some(line_curve(l, opposite)?)
            }
            SurfaceType::Cone => {
                let cone = s.cone()?;
                let mut l = elslib::cone_u_iso(
                    cone.position(),
                    cone.ref_radius(),
                    cone.semi_angle(),
                    p.x(),
                );
                l.translate(Vector3::from(l.direction()) * p.y());
                Some(line_curve(l, opposite)?)
            }
            SurfaceType::Torus => {
                let tore = s.torus()?;
                let axis = tore.position();
                let mut ci =
                    elslib::torus_u_iso(axis, tore.major_radius(), tore.minor_radius(), p.x());
                ci.rotate(ci.axis(), p.y())?;
                Some(circle_curve(ci, opposite)?)
            }
            _ => None,
        };
        if let Some(curve3d) = curve3d {
            update_edge_curve(edge, &curve3d, loc, tol)?;
        }
    }
    Ok(())
}
