// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ProjLib_ComputeApproxOnPolarSurface::BuildInitialCurve2d
// (sampling, analytic surfaces and the resulting curve)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BuildInitialCurve2d`: a degree-1 B-spline through the projections of 61
//! quasi-uniform points of the curve. Analytic surfaces take the ElSLib parameters
//! unwrapped across the seams; the others a chain of extremum searches.
use super::ComputeApproxOnPolarSurface;
use super::function::{compute_periodicity, surface_bounds};
use crate::gcpnts::QuasiUniformAbscissa;
use crate::native_failure::failure;
use crate::proj_lib::new_array;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{self, Point2, Point3, Vector2};
use std::f64::consts::PI;

/// The sampled curve and the data every projection step shares.
pub(super) struct Sampling<'a> {
    pub(super) surf: &'a dyn Adaptor3dSurface,
    pub(super) pts: Array1<'static, Point3>,
    pub(super) param: Array1<'static, f64>,
    pub(super) pts2d: Array1<'static, Point2>,
    pub(super) nb_of_pnts: i32,
    pub(super) tol_u: f64,
    pub(super) tol_v: f64,
    pub(super) dist_tol3d2: f64,
    pub(super) uperiod: f64,
    pub(super) vperiod: f64,
    pub(super) bounds: [f64; 4],
}

impl ComputeApproxOnPolarSurface {
    /// `BuildInitialCurve2d(Curve, S)`: the initial 2D curve, also stored as the
    /// B-spline result; a null handle when the projection fails. Standard_NoSuchObject
    /// for a plane or another surface that is neither elementary nor a Bezier,
    /// B-spline or offset surface; `polar.build_initial_curve2d(&curve, &surface)?`.
    pub fn build_initial_curve2d(
        &mut self,
        curve: &Handle<dyn Adaptor3dCurve>,
        surface: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        self.proj_is_done = false;
        let (c, s) = (super::live_curve(curve)?, super::live_surface(surface)?);
        let mut sampling: Sampling<'_> = self.sample(c, s)?;
        let kind: SurfaceType = s.get_type()?;
        if matches!(
            kind,
            SurfaceType::BSplineSurface | SurfaceType::BezierSurface | SurfaceType::OffsetSurface
        ) {
            self.proj_is_done = false;
            self.project_by_extrema(&mut sampling)?;
        } else {
            analytic_points(&mut sampling, kind)?;
            self.proj_is_done = true;
        }
        if !self.proj_is_done {
            return Ok(Handle::default());
        }
        self.initial_curve(&sampling)
    }

    // The quasi-uniform samples and the tolerances of the projection.
    fn sample<'a>(
        &self,
        curve: &'a dyn Adaptor3dCurve,
        surf: &'a dyn Adaptor3dSurface,
    ) -> Result<Sampling<'a>, Failure> {
        let tol3d: f64 = self.tolerance;
        let (tol_u, tol_v) = (surf.u_resolution(tol3d)?, surf.v_resolution(tol3d)?);
        let dist_tol3d: f64 = if self.max_dist > 0. { self.max_dist } else { 100.0 * tol3d };
        let (uperiod, vperiod) = compute_periodicity(surf)?;
        let qua = QuasiUniformAbscissa::with_nb_points(curve, 61)?;
        let nb_of_pnts: i32 = qua.nb_points()?;
        let mut pts: Array1<'static, Point3> = new_array(nb_of_pnts)?;
        let mut param: Array1<'static, f64> = new_array(nb_of_pnts)?;
        for i in 1..=nb_of_pnts {
            param[i] = qua.parameter(i)?;
            pts[i] = curve.value(param[i])?;
        }
        Ok(Sampling {
            surf,
            pts,
            param,
            pts2d: new_array(nb_of_pnts)?,
            nb_of_pnts,
            tol_u,
            tol_v,
            dist_tol3d2: dist_tol3d * dist_tol3d,
            uperiod,
            vperiod,
            bounds: surface_bounds(surf)?,
        })
    }

    // The degree-1 B-spline through the points, translated by whole periods so
    // that its middle lies in the surface bounds.
    fn initial_curve(&mut self, s: &Sampling<'_>) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let mut mult: Array1<'static, i32> = new_array(s.nb_of_pnts)?;
        for i in 1..=s.nb_of_pnts {
            mult[i] = 1;
        }
        mult[1] = 2;
        mult[s.nb_of_pnts] = 2;
        let spline = Geom2dBSplineCurve::new(&s.pts2d, &s.param, &mult, 1, false)?;
        let mid: Point2 = spline.value(0.5 * (spline.first_parameter() + spline.last_parameter()))?;
        let [uinf, usup, vinf, vsup] = s.bounds;
        let test_u: f64 = into_range(mid.x(), uinf - s.tol_u, usup + s.tol_u, s.uperiod);
        let test_v: f64 = into_range(mid.y(), vinf - s.tol_v, vsup + s.tol_v, s.vperiod);
        let offset: Vector2 = Vector2::new(test_u - mid.x(), test_v - mid.y());
        if offset.x().abs() > gp::resolution() || offset.y().abs() > gp::resolution() {
            spline.translate(offset)?;
        }
        let spline: Handle<Geom2dBSplineCurve> = Handle::new(spline);
        self.bspline = spline.clone();
        let adaptor = Geom2dAdaptorCurve::with_curve(&spline.cast::<dyn Geom2dCurve>())?;
        Ok(Handle::new(adaptor).cast())
    }
}

// `x` moved by whole periods into `[low, high]`; unchanged when not periodic.
fn into_range(mut x: f64, low: f64, high: f64, period: f64) -> f64 {
    if period == 0. {
        return x;
    }
    let mut sense: f64 = 0.;
    if x < low {
        sense = 1.;
    } else if x > high {
        sense = -1.;
    }
    while x < low || x > high {
        x += sense * period;
    }
    x
}

/// The unwrapped ElSLib parameters on a cylinder, cone, sphere or torus;
/// Standard_NoSuchObject for any other analytic surface (a plane included).
fn analytic_points(s: &mut Sampling<'_>, kind: SurfaceType) -> Result<(), Failure> {
    let params = |point: Point3| -> Result<(f64, f64), Failure> {
        Ok(match kind {
            SurfaceType::Cylinder => elslib::parameters(&s.surf.cylinder()?, point),
            SurfaceType::Cone => elslib::parameters(&s.surf.cone()?, point),
            SurfaceType::Sphere => elslib::parameters(&s.surf.sphere()?, point),
            SurfaceType::Torus => elslib::parameters(&s.surf.torus()?, point),
            _ => {
                return Err(failure(
                    "Standard_NoSuchObject",
                    "ProjLib_ComputeApproxOnPolarSurface::BuildInitialCurve2d",
                ));
            }
        })
    };
    let mut uv: Vec<(f64, f64)> = Vec::with_capacity(s.nb_of_pnts as usize);
    for i in 1..=s.nb_of_pnts {
        uv.push(params(s.pts[i])?);
    }
    let unwrapped: Vec<Point2> = match kind {
        SurfaceType::Sphere => unwrap_sphere(&uv, s.bounds),
        SurfaceType::Torus => unwrap_torus(&uv),
        _ => unwrap_u(&uv),
    };
    for (index, point) in unwrapped.into_iter().enumerate() {
        s.pts2d[index as i32 + 1] = point;
    }
    Ok(())
}

// A jump of the angle by more than PI between neighbours crosses the seam.
fn turn_step(previous: f64, current: f64) -> i32 {
    if (previous - current).abs() <= PI {
        return 0;
    }
    if previous > current { 1 } else { -1 }
}

// Cylinder and cone: U unwrapped by whole turns, V as is.
fn unwrap_u(uv: &[(f64, f64)]) -> Vec<Point2> {
    let mut usens: i32 = 0;
    let mut points: Vec<Point2> = vec![Point2::new(uv[0].0, uv[0].1)];
    for pair in uv.windows(2) {
        usens += turn_step(pair[0].0, pair[1].0);
        let (s, t) = pair[1];
        points.push(Point2::new(s + f64::from(usens * 2) * PI, t));
    }
    points
}

// Torus: both angles unwrapped by whole turns.
fn unwrap_torus(uv: &[(f64, f64)]) -> Vec<Point2> {
    let (mut usens, mut vsens): (i32, i32) = (0, 0);
    let mut points: Vec<Point2> = vec![Point2::new(uv[0].0, uv[0].1)];
    for pair in uv.windows(2) {
        usens += turn_step(pair[0].0, pair[1].0);
        vsens += turn_step(pair[0].1, pair[1].1);
        let (s, t) = pair[1];
        points.push(Point2::new(s + f64::from(usens * 2) * PI, t + f64::from(vsens * 2) * PI));
    }
    points
}

// Sphere: U steps by half turns; a jump of about half a turn goes over a pole,
// which mirrors V (`vparit`) and moves it by PI up or down.
fn unwrap_sphere(uv: &[(f64, f64)], bounds: [f64; 4]) -> Vec<Point2> {
    let (vinf, vsup) = (bounds[2], bounds[3]);
    let (mut usens, mut vsens): (i32, i32) = (0, 0);
    let mut vparit: bool = false;
    let mut points: Vec<Point2> = vec![Point2::new(uv[0].0, uv[0].1)];
    for pair in uv.windows(2) {
        let ((sloc, tloc), (s, t)) = (pair[0], pair[1]);
        let jump: f64 = (sloc - s).abs();
        if 1.6 * PI < jump {
            usens += if sloc > s { 2 } else { -2 };
        }
        if 1.6 * PI > jump && jump > 0.4 * PI {
            vparit = !vparit;
            usens += if sloc > s { 1 } else { -1 };
            vsens += if (tloc - vsup).abs() < (vsup - vinf) / 5. { 1 } else { -1 };
        }
        let u: f64 = s + f64::from(usens) * PI;
        let v: f64 = if vparit { (PI - t) * f64::from(vsens - 1) } else { t + f64::from(vsens) * PI };
        points.push(Point2::new(u, v));
    }
    points
}
