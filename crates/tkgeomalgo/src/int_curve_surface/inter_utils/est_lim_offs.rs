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
// Rust translation of EstLimForInfOffs (IntCurveSurface_InterUtils.pxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `EstLimForInfOffs`: the finite parameter box a line can meet on an infinite
//! offset surface, from the equivalent elementary surface or the basis surface.
use super::est_lim::est_lim_for_inf_extr;
use super::{InfFlags, THE_TOLERANCE_ANGULAIRE, THE_TOLTANGENCY, UvLimits};
use crate::int_curve_surface::tools::InterSurfaceTool;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeombase::int_ana::{IntConicQuad, Quadric};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Axis1, Axis3, Cone3, Cylinder3, Line3, Plane3, Point3, Vector3};
use std::f64::consts::PI;

/// `EstLimForInfOffs`: bounds for an offset surface the line can meet; returns
/// NoIntersection. An offset plane, cylinder or cone is intersected as the
/// equivalent elementary surface (10 around the plane point, the widened V range of
/// the meeting points otherwise); an offset extrusion uses the extrusion bounds,
/// then caps U by the kind of its basis curve; an offset revolution caps V by the
/// kind of its basis curve; any other basis caps V to 1e10;
/// `est_lim_for_inf_offs::<HSurfaceTool>(line, &s, 10, inf, &mut limits)?`.
pub fn est_lim_for_inf_offs<S: InterSurfaceTool>(
    line: Line3,
    surface: &S::Surface,
    nbsu: i32,
    inf: InfFlags,
    limits: &mut UvLimits,
) -> Result<bool, Failure> {
    let bas_surf: Adaptor3dSurfaceHandle = S::basis_surface(surface)?;
    let off_val: f64 = S::offset_value(surface)?;
    match bas_surf.get_type()? {
        SurfaceType::Plane => offset_plane(line, bas_surf.plane()?, off_val, limits),
        SurfaceType::Cylinder => offset_cylinder(line, bas_surf.cylinder()?, off_val, limits),
        SurfaceType::Cone => offset_cone(line, bas_surf.cone()?, off_val, limits),
        SurfaceType::SurfaceOfExtrusion => {
            offset_extrusion::<S>(line, surface, &bas_surf, nbsu, inf, limits)
        }
        SurfaceType::SurfaceOfRevolution => {
            let (lo, hi) = cap_of_basis(&bas_surf)?;
            limits.v1 = std_max(limits.v1, lo);
            limits.v2 = std_min(limits.v2, hi);
            Ok(false)
        }
        _ => {
            limits.v1 = std_max(limits.v1, -1.0e10);
            limits.v2 = std_min(limits.v2, 1.0e10);
            Ok(false)
        }
    }
}

/// The meeting points of the line with an elementary quadric: `None` when the
/// intersection is not done, an empty list for a parallel line or one lying on it
/// (NoIntersection).
fn meetings(inter: &IntConicQuad) -> Result<Option<Vec<Point3>>, Failure> {
    if !inter.is_done() {
        return Ok(None);
    }
    if inter.is_parallel()? || inter.is_in_quadric()? {
        return Ok(Some(Vec::new()));
    }
    let points: Result<Vec<Point3>, Failure> =
        (1..=inter.nb_points()?).map(|i| inter.point(i)).collect();
    Ok(Some(points?))
}

/// The plane moved by the offset along its normal: the box of 10 around the
/// parameters of the meeting point.
fn offset_plane(
    line: Line3,
    mut pln: Plane3,
    off_val: f64,
    limits: &mut UvLimits,
) -> Result<bool, Failure> {
    let position: Axis3 = pln.position();
    let mut t = Vector3::from(position.x_direction().crossed(position.y_direction())?);
    t *= off_val;
    pln.translate(t);
    let inter = IntConicQuad::line_plane(line, pln, THE_TOLERANCE_ANGULAIRE, 0.0, 0.0)?;
    let Some(points) = meetings(&inter)? else {
        return Ok(false);
    };
    let Some(&first) = points.first() else {
        return Ok(true);
    };
    let (u, v) = elslib::plane_parameters(pln.position(), first);
    limits.u1 = std_max(limits.u1, u - 10.0);
    limits.u2 = std_min(limits.u2, u + 10.0);
    limits.v1 = std_max(limits.v1, v - 10.0);
    limits.v2 = std_min(limits.v2, v + 10.0);
    Ok(false)
}

/// Widens and applies the V range of the meeting points, their V read by `v_of`;
/// NoIntersection when none.
fn apply_v_range(
    points: Option<Vec<Point3>>,
    v_of: impl Fn(Point3) -> f64,
    limits: &mut UvLimits,
) -> bool {
    let Some(points) = points else {
        return false;
    };
    if points.is_empty() {
        return true;
    }
    let (mut vmin, mut vmax) = (real_last(), -real_last());
    for p in points {
        let v: f64 = v_of(p);
        vmin = std_min(vmin, v);
        vmax = std_max(vmax, v);
    }
    limits.v1 = std_max(limits.v1, vmin - vmin.abs() - 10.0);
    limits.v2 = std_min(limits.v2, vmax + vmax.abs() + 10.0);
    false
}

/// The cylinder of radius `R + offset` (`R - offset` for an indirect frame), turned
/// half a turn when that radius is negative; NoIntersection when it is null.
fn offset_cylinder(
    line: Line3,
    mut cyl: Cylinder3,
    off_val: f64,
    limits: &mut UvLimits,
) -> Result<bool, Failure> {
    let mut r: f64 = cyl.radius();
    let mut a: Axis3 = cyl.position();
    if a.direct()? {
        r += off_val;
    } else {
        r -= off_val;
    }
    if r >= THE_TOLTANGENCY {
        cyl.set_radius(r)?;
    } else if r <= -THE_TOLTANGENCY {
        a.rotate(Axis1::new(a.location(), a.direction()), PI)?;
        cyl.set_position(a);
        cyl.set_radius(-r)?;
    } else {
        return Ok(true);
    }
    let inter = IntConicQuad::line_quadric(line, &Quadric::from_cylinder(cyl))?;
    let v_of = |p: Point3| elslib::cylinder_parameters(cyl.position(), cyl.radius(), p).1;
    Ok(apply_v_range(meetings(&inter)?, v_of, limits))
}

/// The cone of reference radius `R + offset * cos(angle)`, moved by
/// `-offset * sin(angle)` along its axis; nothing for a negative radius.
fn offset_cone(
    line: Line3,
    mut con: Cone3,
    off_val: f64,
    limits: &mut UvLimits,
) -> Result<bool, Failure> {
    let ang: f64 = con.semi_angle();
    let r: f64 = con.ref_radius() + off_val * ang.cos();
    let mut a: Axis3 = con.position();
    if r < 0.0 {
        return Ok(false);
    }
    let mut z = Vector3::from(a.direction());
    z *= -off_val * ang.sin();
    a.translate(z);
    con.set_position(a);
    con.set_radius(r)?;
    con.set_semi_angle(ang)?;
    let inter = IntConicQuad::line_quadric(line, &Quadric::from_cone(con))?;
    let v_of = |p: Point3| {
        elslib::cone_parameters(con.position(), con.ref_radius(), con.semi_angle(), p).1
    };
    Ok(apply_v_range(meetings(&inter)?, v_of, limits))
}

/// An offset extrusion: the extrusion bounds, then an infinite U range capped by
/// the kind of the basis curve of the basis surface, from the original U bounds.
fn offset_extrusion<S: InterSurfaceTool>(
    line: Line3,
    surface: &S::Surface,
    bas_surf: &Adaptor3dSurfaceHandle,
    nbsu: i32,
    inf: InfFlags,
    limits: &mut UvLimits,
) -> Result<bool, Failure> {
    let (u1, u2) = (limits.u1, limits.u2);
    if est_lim_for_inf_extr::<S>(line, surface, true, nbsu, inf, limits)? {
        return Ok(true);
    }
    if inf.u1 || inf.u2 {
        let (lo, hi) = cap_of_basis(bas_surf)?;
        limits.u1 = std_max(u1, lo);
        limits.u2 = std_min(u2, hi);
    }
    Ok(false)
}

/// The cap of a swept basis by its curve: 1e10 for a line, `2e5 sqrt(focal)` for a
/// parabola, 30 for a hyperbola, 1e10 otherwise.
fn cap_of_basis(bas_surf: &Adaptor3dSurfaceHandle) -> Result<(f64, f64), Failure> {
    let curve = bas_surf.basis_curve()?;
    Ok(match curve.get_type()? {
        CurveType::Parabola => {
            let d: f64 = 2.0e5 * curve.parabola()?.focal().sqrt();
            (-d, d)
        }
        CurveType::Hyperbola => (-30.0, 30.0),
        _ => (-1.0e10, 1.0e10),
    })
}
