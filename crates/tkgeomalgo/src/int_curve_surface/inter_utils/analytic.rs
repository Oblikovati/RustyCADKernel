// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ComputeTransitions, ComputeParamsOnQuadric, ComputeAppendPoint,
// ProcessIntAna, PerformCurveQuadric and ProcessLinTorus (IntCurveSurface_InterUtils.pxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection points of the analytic and quadric paths: the transition, the
//! surface parameters and the domain check of each point.
use super::{THE_TOLERANCE_ANGULAIRE, THE_TOLTANGENCY};
use crate::int_curve_surface::quad_curv::QuadCurvExact;
use crate::int_curve_surface::tools::{InterCurveTool, InterSurfaceTool};
use crate::int_curve_surface::{IntersectionPoint, TransitionOnCurve};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana::{IntConicQuad, IntLinTorus};
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Line3, Point3, Vector3};

/// `ComputeTransitions`: In when the curve tangent opposes the surface normal
/// `D1U ^ D1V` beyond the angular tolerance, Out when it follows it, Tangent
/// otherwise or for a null normal or tangent;
/// `compute_transitions::<TheHCurveTool, HSurfaceTool>(&c, 0.5, &s, 0.1, 0.2)?`.
pub fn compute_transitions<C: InterCurveTool, S: InterSurfaceTool>(
    curve: &C::Curve,
    w: f64,
    surface: &S::Surface,
    u: f64,
    v: f64,
) -> Result<TransitionOnCurve, Failure> {
    let d1 = S::d1(surface, u, v)?;
    let n_surf: Vector3 = d1.d1u.crossed(d1.d1v);
    let (_, mut d1w): (Point3, Vector3) = C::d1(curve, w)?;
    let norm: f64 = n_surf.magnitude();
    if !(norm > THE_TOLERANCE_ANGULAIRE && d1w.square_magnitude() > THE_TOLERANCE_ANGULAIRE) {
        return Ok(TransitionOnCurve::Tangent);
    }
    d1w.normalize()?;
    let mut cos_dir: f64 = n_surf.dot(d1w);
    cos_dir /= norm;
    Ok(if -cos_dir > THE_TOLERANCE_ANGULAIRE {
        TransitionOnCurve::In
    } else if cos_dir > THE_TOLERANCE_ANGULAIRE {
        TransitionOnCurve::Out
    } else {
        TransitionOnCurve::Tangent
    })
}

/// `ComputeParamsOnQuadric`: the parameters of `p` on a plane, cylinder, cone or
/// sphere; `u` and `v` are left unchanged for any other surface, as natively;
/// `compute_params_on_quadric::<HSurfaceTool>(&s, p, &mut u, &mut v)?`.
pub fn compute_params_on_quadric<S: InterSurfaceTool>(
    surface: &S::Surface,
    p: Point3,
    u: &mut f64,
    v: &mut f64,
) -> Result<(), Failure> {
    (*u, *v) = match S::get_type(surface)? {
        SurfaceType::Plane => elslib::plane_parameters(S::plane(surface)?.position(), p),
        SurfaceType::Cylinder => {
            let c = S::cylinder(surface)?;
            elslib::cylinder_parameters(c.position(), c.radius(), p)
        }
        SurfaceType::Cone => {
            let c = S::cone(surface)?;
            elslib::cone_parameters(c.position(), c.ref_radius(), c.semi_angle(), p)
        }
        SurfaceType::Sphere => {
            let s = S::sphere(surface)?;
            elslib::sphere_parameters(s.position(), s.radius(), p)
        }
        _ => return Ok(()),
    };
    Ok(())
}

/// `ComputeAppendPoint`: the intersection point at `(lw, su, sv)` with the curve
/// and the U parameters brought into their period, or `None` when a parameter lies
/// beyond the curve or surface range by `THE_TOLTANGENCY`;
/// `compute_append_point::<TheHCurveTool, HSurfaceTool>(&c, 0.5, &s, 0.1, 0.2)?`.
pub fn compute_append_point<C: InterCurveTool, S: InterSurfaceTool>(
    curve: &C::Curve,
    lw: f64,
    surface: &S::Surface,
    su: f64,
    sv: f64,
) -> Result<Option<IntersectionPoint>, Failure> {
    let (w0, w1) = (C::first_parameter(curve)?, C::last_parameter(curve)?);
    let (u0, u1) = (
        S::first_u_parameter(surface)?,
        S::last_u_parameter(surface)?,
    );
    let (v0, v1) = (
        S::first_v_parameter(surface)?,
        S::last_v_parameter(surface)?,
    );
    let (mut w, mut u, mut v) = (lw, su, sv);
    let c_type: CurveType = C::get_type(curve)?;
    if C::is_periodic(curve)? || c_type == CurveType::Circle || c_type == CurveType::Ellipse {
        w = elclib::in_period(w, w0, w0 + C::period(curve)?);
    }
    if (w0 - w) >= THE_TOLTANGENCY || (w - w1) >= THE_TOLTANGENCY {
        return Ok(None);
    }
    let s_type: SurfaceType = S::get_type(surface)?;
    let revolving = matches!(
        s_type,
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere
    );
    if S::is_u_periodic(surface)? || revolving {
        u = elclib::in_period(u, u0, u0 + S::u_period(surface)?);
    }
    if S::is_v_periodic(surface)? {
        v = elclib::in_period(v, v0, v0 + S::v_period(surface)?);
    }
    if (u0 - u) >= THE_TOLTANGENCY || (u - u1) >= THE_TOLTANGENCY {
        return Ok(None);
    }
    if (v0 - v) >= THE_TOLTANGENCY || (v - v1) >= THE_TOLTANGENCY {
        return Ok(None);
    }
    let transition: TransitionOnCurve = compute_transitions::<C, S>(curve, w, surface, u, v)?;
    let p: Point3 = C::value(curve, w)?;
    Ok(Some(IntersectionPoint::new(p, u, v, w, transition)))
}

/// The points an analytic intersection gives: `None` when it is not done,
/// `Some(None)` when the curve is parallel to the surface or lies on it, else the
/// points kept by [`compute_append_point`].
pub type IntAnaPoints = Option<Option<Vec<IntersectionPoint>>>;

/// `ProcessIntAna`: the points of a conic-quadric intersection, each with its
/// parameters on the quadric; see [`IntAnaPoints`];
/// `process_int_ana::<TheHCurveTool, HSurfaceTool>(&c, &s, &inter)?`.
pub fn process_int_ana<C: InterCurveTool, S: InterSurfaceTool>(
    curve: &C::Curve,
    surface: &S::Surface,
    int_ana: &IntConicQuad,
) -> Result<IntAnaPoints, Failure> {
    if !int_ana.is_done() {
        return Ok(None);
    }
    if int_ana.is_in_quadric()? || int_ana.is_parallel()? {
        return Ok(Some(None));
    }
    // `u` and `v` live across the loop natively; they are always written here, since
    // the analytic paths only run on quadrics.
    let (mut u, mut v) = (0.0, 0.0);
    let mut points: Vec<IntersectionPoint> = Vec::new();
    for i in 1..=int_ana.nb_points()? {
        let p: Point3 = int_ana.point(i)?;
        let w: f64 = int_ana.param_on_conic(i)?;
        compute_params_on_quadric::<S>(surface, p, &mut u, &mut v)?;
        points.extend(compute_append_point::<C, S>(curve, w, surface, u, v)?);
    }
    Ok(Some(Some(points)))
}

/// `PerformCurveQuadric`: the points at the roots of the curve-quadric distance,
/// with their parameters on the quadric (none when the root search is not done);
/// `perform_curve_quadric::<TheHCurveTool, HSurfaceTool>(&c, &s)?`.
pub fn perform_curve_quadric<C: InterCurveTool, S: InterSurfaceTool>(
    curve: &C::Curve,
    surface: &S::Surface,
) -> Result<Vec<IntersectionPoint>, Failure> {
    let quad_curv: QuadCurvExact<S, C> = QuadCurvExact::new(surface, curve)?;
    let mut points: Vec<IntersectionPoint> = Vec::new();
    if !quad_curv.is_done() {
        return Ok(points);
    }
    let (mut u, mut v) = (0.0, 0.0);
    for i in 1..=quad_curv.nb_roots() {
        let w: f64 = quad_curv.root(i)?;
        compute_params_on_quadric::<S>(surface, C::value(curve, w)?, &mut u, &mut v)?;
        points.extend(compute_append_point::<C, S>(curve, w, surface, u, v)?);
    }
    Ok(points)
}

/// `ProcessLinTorus`: the points of a line-torus intersection, `None` when it is
/// not done (the caller then samples the torus);
/// `process_lin_torus::<TheHCurveTool, HSurfaceTool>(line, &c, &s)?`.
pub fn process_lin_torus<C: InterCurveTool, S: InterSurfaceTool>(
    line: Line3,
    curve: &C::Curve,
    surface: &S::Surface,
) -> Result<Option<Vec<IntersectionPoint>>, Failure> {
    let inter = IntLinTorus::with(line, S::torus(surface)?)?;
    if !inter.is_done() {
        return Ok(None);
    }
    let mut points: Vec<IntersectionPoint> = Vec::new();
    for i in 1..=inter.nb_points()? {
        let w: f64 = inter.param_on_line(i)?;
        let (fi, theta) = inter.param_on_torus(i)?;
        points.extend(compute_append_point::<C, S>(curve, w, surface, fi, theta)?);
    }
    Ok(Some(points))
}
