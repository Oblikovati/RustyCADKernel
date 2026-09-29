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
// Rust translation of the plane cases of ProjLib_ComputeApprox::Perform and of its
// file-local PlaneProjector at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A Bezier or B-spline curve on a plane: the 2D curve with the plane coordinates
//! of its poles, the same weights and knots.
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBezierCurve};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurve};
use rustycad_tkmath::gp::{Plane3, Point2, Point3};

/// The file-local `PlaneProjector`: the plane coordinates of a point with the
/// basis read once (no gp_Trsf per point).
struct PlaneProjector {
    origin: [f64; 3],
    dx: [f64; 3],
    dy: [f64; 3],
}

impl PlaneProjector {
    fn new(plane: &Plane3) -> Self {
        let position = plane.position();
        let (o, x, y) = (position.location(), position.x_direction(), position.y_direction());
        Self {
            origin: [o.x(), o.y(), o.z()],
            dx: [x.x(), x.y(), x.z()],
            dy: [y.x(), y.y(), y.z()],
        }
    }
    // X = (P - O) . XDirection, Y = (P - O) . YDirection, summed left to right.
    fn project(&self, p: Point3) -> Point2 {
        let (dx, dy, dz) = (p.x() - self.origin[0], p.y() - self.origin[1], p.z() - self.origin[2]);
        Point2::new(
            dx * self.dx[0] + dy * self.dx[1] + dz * self.dx[2],
            dx * self.dy[0] + dy * self.dy[1] + dz * self.dy[2],
        )
    }
    fn project_all(&self, poles: &Array1<'_, Point3>) -> Result<Array1<'static, Point2>, Failure> {
        let mut projected: Array1<'static, Point2> = super::super::new_array(poles.len() as i32)?;
        for i in 1..=poles.len() as i32 {
            projected[i] = self.project(poles[i]);
        }
        Ok(projected)
    }
}

fn null_spline(member: &str) -> Failure {
    failure("Standard_NullObject", &format!("ProjLib_ComputeApprox: {member} returned a null curve"))
}

/// The B-spline case: the whole B-spline of the adaptor (not its trimmed range),
/// with its poles projected.
pub(super) fn bspline_on_plane(
    curve: &dyn Adaptor3dCurve,
    plane: &Plane3,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let source: Handle<GeomBSplineCurve> = curve.bspline()?;
    let bs: &GeomBSplineCurve = source.get().ok_or_else(|| null_spline("BSpline()"))?;
    let poles: Array1<'static, Point2> = PlaneProjector::new(plane).project_all(&bs.poles())?;
    let (knots, mults) = (bs.knots(), bs.multiplicities());
    let (degree, periodic) = (bs.degree(), bs.is_periodic());
    let spline: Geom2dBSplineCurve = if bs.is_rational() {
        let weights: Array1<'static, f64> = bs.weights_array(|w| w.clone());
        Geom2dBSplineCurve::with_weights(&poles, &weights, &knots, &mults, degree, periodic)?
    } else {
        Geom2dBSplineCurve::new(&poles, &knots, &mults, degree, periodic)?
    };
    Ok(Handle::new(spline))
}

/// The Bezier case: the Bezier curve of the adaptor with its poles projected.
pub(super) fn bezier_on_plane(
    curve: &dyn Adaptor3dCurve,
    plane: &Plane3,
) -> Result<Handle<Geom2dBezierCurve>, Failure> {
    let source: Handle<GeomBezierCurve> = curve.bezier()?;
    let bc: &GeomBezierCurve = source.get().ok_or_else(|| null_spline("Bezier()"))?;
    let poles: Array1<'static, Point2> = PlaneProjector::new(plane).project_all(&bc.poles())?;
    let bezier: Geom2dBezierCurve = if bc.is_rational() {
        let weights: Array1<'static, f64> = bc.weights_array(|w| w.clone());
        Geom2dBezierCurve::with_weights(&poles, &weights)?
    } else {
        Geom2dBezierCurve::new(&poles)?
    };
    Ok(Handle::new(bezier))
}
