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
// Rust translation of the "Return curve home" step of ProjLib_ComputeApprox::Perform
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! "Return curve home": the fitted curve is moved by whole periods (and mirrored
//! across V = 0 on a sphere) so that its middle point matches the ElSLib parameters
//! of the curve middle.
use super::ComputeApprox;
use super::function::{Projection, ProjectionFunction, elementary_parameters};
use crate::app_cont::Function;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::Geom2dGeometry;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, StandardDirection2, Vector2};
use std::f64::consts::PI;

/// The translation (and mirror) that brings the fitted curve home.
struct HomeShift {
    du: f64,
    dv: f64,
    to_mirror: bool,
}

// The V shift: whole V windows, after the sphere mirror when U is more than PI away.
fn v_shift(f: &ProjectionFunction<'_>, sphere: bool, u: f64, mut v: f64) -> HomeShift {
    let b = &f.bounds;
    let mut shift = HomeShift { du: 0., dv: 0., to_mirror: false };
    if sphere && (u - b.u1).abs() > PI {
        shift.to_mirror = true;
        shift.dv = -PI;
        v = PI - v;
    }
    let new_v: f64 = elclib::in_period(v, b.v1, b.v2);
    let number: i32 = ((new_v - v) / (b.v2 - b.v1)).floor() as i32;
    shift.dv -= f64::from(number) * (b.v2 - b.v1);
    shift
}

// The U shift: whole multiples of PI between the curve middle and the fitted
// function middle, rounded away from zero by PConfusion.
fn u_shift(f: &ProjectionFunction<'_>, u: f64, umid: f64) -> Result<f64, Failure> {
    let p2d: Point2 = f.point(umid)?;
    let mut du: f64 = u - p2d.x();
    du = if du < 0. { du - precision::p_confusion() } else { du + precision::p_confusion() };
    let number: i32 = (du / PI).trunc() as i32;
    Ok(f64::from(number) * PI)
}

/// Moves `approx.bspline` home; Standard_NoSuchObject for a non-elementary surface.
pub(super) fn bring_home(
    approx: &mut ComputeApprox,
    f: &ProjectionFunction<'_>,
    projection: Projection<'_>,
) -> Result<(), Failure> {
    let umid: f64 = (f.first_parameter() + f.last_parameter()) / 2.;
    let p3d = projection.curve.value(umid)?;
    let (u, v) = elementary_parameters(projection.surface, p3d, "ProjLib_ComputeApprox::Value")?;
    let sphere: bool = projection.surface.get_type()? == SurfaceType::Sphere;
    let mut shift = HomeShift { du: 0., dv: 0., to_mirror: false };
    if f.bounds.v_couture {
        shift = v_shift(f, sphere, u, v);
    }
    if f.bounds.u_couture || (f.bounds.v_couture && sphere) {
        shift.du = u_shift(f, u, umid)?;
    }
    let Some(spline) = approx.bspline.get() else {
        return Ok(());
    };
    if shift.du != 0. || shift.dv != 0. {
        spline.translate(Vector2::new(shift.du, shift.dv))?;
    }
    if shift.to_mirror {
        let axis = Axis2d::new(Point2::new(0., 0.), Direction2::from_standard(StandardDirection2::X));
        spline.mirror_axis(axis)?;
    }
    Ok(())
}
