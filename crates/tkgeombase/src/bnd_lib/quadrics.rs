// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the cylinder, cone, sphere and torus members of BndLib.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BndLib::Add` for the elementary surfaces. The boundary circles go through
//! `BndLib::Add(gp_Circ, U1, U2, 0, B)` on the accumulated box.
use super::compute::{ArcFrame, COS_PI4, compute};
use super::conics::add_circ_arc;
use crate::geom_bnd_lib::patch::{Patch, add_axis_extrema, add_iso_circles};
use crate::geom_bnd_lib::ranges::add_v_range;
use crate::geom_bnd_lib::sphere_patch;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Circle3, Cone3, Cylinder3, Point3, Sphere3, Torus3, Vector3, Xyz};
use std::f64::consts::PI;

// The native message of every BndLib `bad parameter` raise.
const BAD_PARAMETER: &str = "BndLib::bad parameter";

// `BndLib::Add(C, U1, U2, 0., B)` as the patch arc adder.
fn add_arc(
    bounds: &mut BoundingBox3,
    circle: &Circle3,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    add_circ_arc(circle, first, last, 0.0, bounds)
}

/// `BndLib::Add(gp_Cylinder, UMin, UMax, VMin, VMax, Tol, B)`: the circles at both ends of
/// the V band; an infinite V end uses V = 0 and opens the axis on its side; a doubly
/// infinite V range only opens the axis; two infinite ends of the same sign raise
/// Standard_Failure; `add_cylinder_patch(&cylinder, [0.0, 6.3, 0.0, 10.0], 0.0, &mut bounds)?`.
pub fn add_cylinder_patch(
    cylinder: &Cylinder3,
    [u_min, u_max, v_min, v_max]: [f64; 4],
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let mut band = |b: &mut BoundingBox3, v1: f64, v2: f64| {
        let mut circle: Circle3 =
            elslib::cylinder_v_iso(cylinder.position(), cylinder.radius(), v1);
        add_circ_arc(&circle, u_min, u_max, 0.0, b)?;
        circle.translate(Vector3::from(cylinder.axis().direction()) * (v2 - v1));
        add_circ_arc(&circle, u_min, u_max, 0.0, b)
    };
    let mut whole = |_: &mut BoundingBox3| Ok(());
    let axis = cylinder.axis().direction();
    add_v_range(
        bounds,
        axis,
        (v_min, v_max),
        &mut band,
        &mut whole,
        BAD_PARAMETER,
    )?;
    bounds.enlarge(tolerance);
    Ok(())
}

/// `BndLib::Add(gp_Cylinder, VMin, VMax, Tol, B)`: the whole U period;
/// `add_cylinder(&cylinder, 0.0, 10.0, 0.0, &mut bounds)?`.
pub fn add_cylinder(
    cylinder: &Cylinder3,
    v_min: f64,
    v_max: f64,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    add_cylinder_patch(cylinder, [0.0, 2.0 * PI, v_min, v_max], tolerance, bounds)
}

// `ComputeCone`'s circle at `v`: an arc, or the apex point below Precision::Confusion().
fn add_cone_circle(
    cone: &Cone3,
    v: f64,
    (u_min, u_max): (f64, f64),
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let circle: Circle3 =
        elslib::cone_v_iso(cone.position(), cone.ref_radius(), cone.semi_angle(), v);
    if circle.radius() > precision::confusion() {
        return add_circ_arc(&circle, u_min, u_max, 0.0, bounds);
    }
    bounds.add_point(circle.location());
    Ok(())
}

/// `BndLib::Add(gp_Cone, UMin, UMax, VMin, VMax, Tol, B)`: the V iso circles at both
/// ends of the band (the apex as a point); infinite V as for the cylinder;
/// `add_cone_patch(&cone, [0.0, 6.3, 0.0, 5.0], 0.0, &mut bounds)?`.
pub fn add_cone_patch(
    cone: &Cone3,
    [u_min, u_max, v_min, v_max]: [f64; 4],
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let mut band = |b: &mut BoundingBox3, v1: f64, v2: f64| {
        add_cone_circle(cone, v1, (u_min, u_max), b)?;
        add_cone_circle(cone, v2, (u_min, u_max), b)
    };
    let mut whole = |_: &mut BoundingBox3| Ok(());
    let axis = cone.axis().direction();
    add_v_range(
        bounds,
        axis,
        (v_min, v_max),
        &mut band,
        &mut whole,
        BAD_PARAMETER,
    )?;
    bounds.enlarge(tolerance);
    Ok(())
}

/// `BndLib::Add(gp_Cone, VMin, VMax, Tol, B)`: the whole U period;
/// `add_cone(&cone, 0.0, 5.0, 0.0, &mut bounds)?`.
pub fn add_cone(
    cone: &Cone3,
    v_min: f64,
    v_max: f64,
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    add_cone_patch(cone, [0.0, 2.0 * PI, v_min, v_max], tolerance, bounds)
}

/// `BndLib::Add(gp_Sphere, Tol, B)`: the cube `centre +/- R`; `add_sphere(&sphere, 0.0, &mut bounds)`.
pub fn add_sphere(sphere: &Sphere3, tolerance: f64, bounds: &mut BoundingBox3) {
    let e: [f64; 6] = crate::geom_bnd_lib::sphere_extents(sphere);
    bounds.update_bounds([e[0], e[2], e[4]], [e[1], e[3], e[5]]);
    bounds.enlarge(tolerance);
}

/// `BndLib::Add(gp_Sphere, UMin, UMax, VMin, VMax, Tol, B)`: `ComputeSphere` (the whole
/// cube when both periods are spanned, else the axis extremes in the patch and the
/// boundary circles); `add_sphere_patch(&sphere, [0.0, 6.3, 0.0, 1.5], 0.0, &mut bounds)`.
pub fn add_sphere_patch(
    sphere: &Sphere3,
    [u_min, u_max, v_min, v_max]: [f64; 4],
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    sphere_patch(
        bounds,
        sphere,
        Patch {
            u_min,
            u_max,
            v_min,
            v_max,
        },
        &add_arc,
    )?;
    bounds.enlarge(tolerance);
    Ok(())
}

/// `BndLib::Add(gp_Torus, Tol, B)`: the eight corners `O +/- (R + r) Xd +/- (R + r) Yd
/// +/- r Zd`; `add_torus(&torus, 0.0, &mut bounds)`.
pub fn add_torus(torus: &Torus3, tolerance: f64, bounds: &mut BoundingBox3) {
    let position = torus.position();
    let reach: f64 = torus.major_radius() + torus.minor_radius();
    let origin: Xyz = position.location().xyz();
    let rx: Xyz = position.x_direction().xyz() * reach;
    let ry: Xyz = position.y_direction().xyz() * reach;
    let rz: Xyz = position.direction().xyz() * torus.minor_radius();
    let corners: [(f64, f64, f64); 8] = [
        (-1.0, -1.0, 1.0),
        (-1.0, -1.0, -1.0),
        (1.0, -1.0, 1.0),
        (1.0, -1.0, -1.0),
        (-1.0, 1.0, 1.0),
        (-1.0, 1.0, -1.0),
        (1.0, 1.0, 1.0),
        (1.0, 1.0, -1.0),
    ];
    let signed =
        |base: Xyz, part: Xyz, sign: f64| if sign > 0.0 { base + part } else { base - part };
    for (sx, sy, sz) in corners {
        let corner: Xyz = signed(signed(signed(origin, rx, sx), ry, sy), rz, sz);
        bounds.add_point(Point3::from(corner));
    }
    bounds.enlarge(tolerance);
}

// `floor(V / (Pi/4))` as `static_cast<int>`: the torus section indices of the V range.
fn section_range(v_min: f64, v_max: f64) -> (i32, i32) {
    use rustycad_tkmath::bvh::BvhScalar;
    let index = |v: f64| (v / (PI / 4.0)).floor().truncate_to_int();
    let (first, last) = if v_max < v_min {
        (index(v_max), index(v_min))
    } else {
        (index(v_min), index(v_max))
    };
    (first, last.wrapping_add(1))
}

/// `BndLib::Add(gp_Torus, UMin, UMax, VMin, VMax, Tol, B)`: for each 45-degree section
/// circle of the V range, the `Compute` polygon of `[UMin, UMax]`; a torus with
/// `R < r` uses the degenerate patch analysis. An empty section range returns without
/// the final Enlarge, as natively; `add_torus_patch(&torus, [0.0, 1.5, 0.0, 3.1], 0.0, &mut bounds)`.
pub fn add_torus_patch(
    torus: &Torus3,
    [u_min, u_max, v_min, v_max]: [f64; 4],
    tolerance: f64,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let (first, last) = section_range(v_min, v_max);
    let (ra, ri) = (torus.major_radius(), torus.minor_radius());
    if last < first {
        return Ok(());
    }
    if ra < ri {
        degenerate_torus_patch(
            torus,
            Patch {
                u_min,
                u_max,
                v_min,
                v_max,
            },
            bounds,
        )?;
        bounds.enlarge(tolerance);
        return Ok(());
    }
    let position = torus.position();
    let (z_dir, location) = (position.direction().xyz(), position.location().xyz());
    let x_dir: Point3 = Point3::from(position.x_direction().xyz());
    let y_dir: Point3 = Point3::from(position.y_direction().xyz());
    let radius_mult: [f64; 8] = [1.0, COS_PI4, 0.0, -COS_PI4, -1.0, -COS_PI4, 0.0, COS_PI4];
    let z_mult: [f64; 8] = [0.0, COS_PI4, 1.0, COS_PI4, 0.0, -COS_PI4, -1.0, -COS_PI4];
    for i in first..=last {
        // `((i % 8) + 8) % 8`: C++ `%` keeps the sign of a negative index.
        let index: usize = ((i % 8 + 8) % 8) as usize;
        let radius: f64 = ra + ri * radius_mult[index];
        let centre: Point3 = Point3::from(location + z_dir * (ri * z_mult[index]));
        let frame = ArcFrame {
            ra: radius,
            rb: radius,
            x_dir,
            y_dir,
            origin: centre,
        };
        compute(bounds, u_min, u_max, &frame);
    }
    bounds.enlarge(tolerance);
    Ok(())
}

// `computeDegeneratedTorus`: a torus with `R < r` (self-intersecting); the whole box
// when the patch spans U and the `2 acos(-R/r)` V range, else the axis extremes in the
// patch and the boundary circles.
fn degenerate_torus_patch(
    torus: &Torus3,
    patch: Patch,
    bounds: &mut BoundingBox3,
) -> Result<(), Failure> {
    let (ra, ri) = (torus.major_radius(), torus.minor_radius());
    let p: Point3 = torus.location();
    let e: [f64; 6] = [
        p.x() - ra - ri,
        p.x() + ra + ri,
        p.y() - ra - ri,
        p.y() + ra + ri,
        p.z() - ri,
        p.z() + ri,
    ];
    let phi: f64 = (-ra / ri).acos();
    let u_period: f64 = 2.0 * PI - precision::p_confusion();
    let v_period: f64 = 2.0 * phi - precision::p_confusion();
    if patch.u_max - patch.u_min >= u_period && patch.v_max - patch.v_min >= v_period {
        bounds.update_bounds([e[0], e[2], e[4]], [e[1], e[3], e[5]]);
        return Ok(());
    }
    let position = torus.position();
    let parameters = |point: Point3| elslib::torus_parameters(position, ra, ri, point);
    add_axis_extrema(bounds, p, e, &parameters, patch);
    let u_iso = |u: f64| elslib::torus_u_iso(position, ra, ri, u);
    let v_iso = |v: f64| elslib::torus_v_iso(position, ra, ri, v);
    add_iso_circles(bounds, &u_iso, &v_iso, patch, &add_arc)
}
