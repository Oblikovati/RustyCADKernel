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
// Rust translation of aFuncStruct, computePeriodicity, anOrthogSqValue, Function_Value
// and ProjLib_PolarFunction (ProjLib_ComputeApproxOnPolarSurface.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The projected curve of ProjLib_ComputeApproxOnPolarSurface as a continuous 2D
//! function: each point starts from the initial 2D curve and is refined by the
//! ElSLib parameters (analytic surfaces) or by a local extremum search on a small
//! patch around it (Bezier, B-spline and offset surfaces).
use crate::app_cont::{Function, FunctionBase};
use crate::extrema::{ExtAlgo, ExtFlag, ExtPS, GenLocateExtPS};
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use crate::proj_lib::compute_approx::function::elementary_parameters;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomSurface, GeomSurfaceD1};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorSurface, make_surface};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::cell::RefCell;
use std::f64::consts::PI;

/// `aFuncStruct`: the data of the projection function.
pub(super) struct FuncData<'a> {
    pub(super) surf: &'a dyn Adaptor3dSurface,
    pub(super) curve: &'a dyn Adaptor3dCurve,
    pub(super) init_curve2d: &'a dyn Adaptor2dCurve2d,
    /// `mySqProjOrtTol`: filters non-orthogonal projected points.
    pub(super) sq_proj_ort_tol: f64,
    pub(super) tol_u: f64,
    pub(super) tol_v: f64,
    /// `myPeriod`: the U and V periods.
    pub(super) period: [f64; 2],
}

/// `computePeriodicity(S, UPeriod, VPeriod)`: the period (or closed range) of each
/// direction, or 0 when the parameter space is not periodic or reduced by a
/// rectangular trim; `compute_periodicity(surface)?`.
pub(in crate::proj_lib) fn compute_periodicity(
    surface: &dyn Adaptor3dSurface,
) -> Result<(f64, f64), Failure> {
    let untrimmed: Handle<dyn GeomSurface> = make_surface(surface, false)?;
    let basis: &dyn GeomSurface = untrimmed.get().ok_or_else(|| {
        failure("Standard_NullObject", "ProjLib_ComputeApproxOnPolarSurface: no Geom surface")
    })?;
    let (base_u1, base_u2, base_v1, base_v2) = basis.bounds();
    let mut u_period: f64 = 0.;
    if surface.is_u_periodic()? {
        u_period = surface.u_period()?;
    } else if surface.is_u_closed()? {
        u_period = surface.last_u_parameter()? - surface.first_u_parameter()?;
    }
    let trim_u = (surface.first_u_parameter()?, surface.last_u_parameter()?);
    u_period = unless_reduced(u_period, trim_u, (base_u1, base_u2));
    let mut v_period: f64 = 0.;
    if surface.is_v_periodic()? {
        v_period = surface.v_period()?;
    } else if surface.is_v_closed()? {
        v_period = surface.last_v_parameter()? - surface.first_v_parameter()?;
    }
    let trim_v = (surface.first_v_parameter()?, surface.last_v_parameter()?);
    v_period = unless_reduced(v_period, trim_v, (base_v1, base_v2));
    Ok((u_period, v_period))
}

// A period is dropped when the trimmed range differs from the untrimmed one.
fn unless_reduced(period: f64, trim: (f64, f64), base: (f64, f64)) -> f64 {
    let pconf: f64 = precision::p_confusion();
    if period.abs() > pconf && (base.0 - trim.0).abs() + (base.1 - trim.1).abs() > pconf {
        return 0.0;
    }
    period
}

/// `anOrthogSqValue(P, S, U, V)`: the squared components of `S(u,v) - P` along the
/// unit tangents (the unnormalized tangent when almost null).
fn orthog_sq_value(base: Point3, surf: &dyn Adaptor3dSurface, u: f64, v: f64) -> Result<f64, Failure> {
    let d: GeomSurfaceD1 = surf.d1(u, v)?;
    let base_vec: Vector3 = Vector3::between(base, d.point);
    let (mut su, mut sv) = (d.d1u, d.d1v);
    if su.square_magnitude() > precision::square_confusion() {
        su.normalize()?;
    }
    if sv.square_magnitude() > precision::square_confusion() {
        sv.normalize()?;
    }
    let (first, second) = (su.dot(base_vec), sv.dot(base_vec));
    Ok(first * first + second * second)
}

// The whole turns to add to leave [inf, sup]: `-int((inf - x) / 2PI) - 1` below,
// `int((x - sup) / 2PI) + 1` above.
fn turns(x: f64, inf: f64, sup: f64) -> i32 {
    let mut decal: i32 = 0;
    if x < inf {
        decal = -(((inf - x) / (2. * PI)) as i32) - 1;
    }
    if x > sup {
        decal = ((x - sup) / (2. * PI)) as i32 + 1;
    }
    decal
}

// `S += decal * 2 * M_PI`, evaluated as the native integer product then the real one.
fn add_turns(value: f64, decal: i32) -> f64 {
    value + f64::from(decal * 2) * PI
}

/// The surface bounds `(Uinf, Usup, Vinf, Vsup)`.
pub(super) fn surface_bounds(surf: &dyn Adaptor3dSurface) -> Result<[f64; 4], Failure> {
    Ok([
        surf.first_u_parameter()?,
        surf.last_u_parameter()?,
        surf.first_v_parameter()?,
        surf.last_v_parameter()?,
    ])
}

impl FuncData<'_> {
    /// `Function_Value(theU, theData)`: the projected point of parameter `u`.
    pub(super) fn value(&self, u: f64) -> Result<Point2, Failure> {
        let mut p2d: Point2 = self.init_curve2d.value(u)?;
        let p: Point3 = self.curve.value(u)?;
        let surf_pnt_dist: f64 = self.surf.value(p2d.x(), p2d.y())?.square_distance(&p);
        let [uinf, usup, vinf, vsup] = surface_bounds(self.surf)?;
        let pconf: f64 = precision::p_confusion();
        // A curve close to a co-parametrized iso line of the surface.
        if (p2d.x() - uinf).abs() < pconf || (p2d.x() - usup).abs() < pconf {
            if self.surf.d0(p2d.x(), u)?.square_distance(&p) < surf_pnt_dist {
                p2d.set_y(u);
            }
        }
        if (p2d.y() - vinf).abs() < pconf || (p2d.y() - vsup).abs() < pconf {
            if self.surf.d0(u, p2d.y())?.square_distance(&p) < surf_pnt_dist {
                p2d.set_x(u);
            }
        }
        let kind: SurfaceType = self.surf.get_type()?;
        if !matches!(
            kind,
            SurfaceType::BSplineSurface | SurfaceType::BezierSurface | SurfaceType::OffsetSurface
        ) {
            return self.analytic_value(kind, p, p2d, [uinf, usup, vinf, vsup]);
        }
        self.patch_value(kind, p, p2d, surf_pnt_dist)
    }

    // The ElSLib parameters of the curve point, moved by whole turns near (U0, V0).
    fn analytic_value(&self, kind: SurfaceType, p: Point3, p2d: Point2, b: [f64; 4]) -> Result<Point2, Failure> {
        let (u0, v0) = (p2d.x(), p2d.y());
        let [uinf, usup, vinf, vsup] = b;
        let member: &str = "ProjLib_ComputeApproxOnPolarSurface::Value";
        if !matches!(kind, SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Torus) {
            return Err(failure("Standard_NoSuchObject", member));
        }
        let (mut s, mut t) = elementary_parameters(self.surf, p, member)?;
        let decal_u: i32 = turns(u0, uinf, usup);
        match kind {
            SurfaceType::Sphere => {
                s = add_turns(s, decal_u);
                t = add_turns(t, sphere_v_turns(v0, vinf, vsup));
                if 0.4 * PI < (u0 - s).abs() && (u0 - s).abs() < 1.6 * PI {
                    t = PI - t;
                    s = if u0 < s { s - PI } else { s + PI };
                }
            }
            SurfaceType::Torus => {
                let decal_v: i32 = turns(v0, vinf, vsup);
                s = add_turns(s, decal_u);
                t = add_turns(t, decal_v);
            }
            _ => s = add_turns(s, decal_u),
        }
        Ok(Point2::new(s, t))
    }
}

// The sphere V turns: the upper test is against `Vsup + (Vsup - Vinf)`.
fn sphere_v_turns(v0: f64, vinf: f64, vsup: f64) -> i32 {
    let mut decal: i32 = 0;
    if v0 < vinf {
        decal = -(((vinf - v0) / (2. * PI)) as i32) - 1;
    }
    if v0 > (vsup + (vsup - vinf)) {
        decal = ((v0 - vsup + (vsup - vinf)) / (2. * PI)) as i32 + 1;
    }
    decal
}

/// `(U0, decal)`: a parameter moved into `[inf, sup]` by whole periods, or clamped
/// when not periodic.
fn into_period(mut x: f64, inf: f64, sup: f64, period: f64) -> (f64, i32) {
    let mut decal: i32 = 0;
    if x < inf {
        if period == 0. {
            x = inf;
        } else {
            decal = ((inf - x) / period) as i32 + 1;
            x += f64::from(decal) * period;
        }
    }
    if x > sup {
        if period == 0. {
            x = sup;
        } else {
            decal = -(((x - sup) / period) as i32 + 1);
            x += f64::from(decal) * period;
        }
    }
    (x, decal)
}

// The sub-range of `[inf, sup]` within a tenth of its length around `x`.
fn little_range(x: f64, inf: f64, sup: f64) -> (f64, f64) {
    let little: f64 = (sup - inf) / 10.;
    let low: f64 = if (x - inf) > little { x - little } else { inf };
    let high: f64 = if (sup - x) > little { x + little } else { sup };
    (low, high)
}

impl FuncData<'_> {
    // The surface patch around (U0, V0) as a GeomAdaptor_Surface.
    fn little_surface(&self, kind: SurfaceType, u: (f64, f64), v: (f64, f64)) -> Result<GeomAdaptorSurface, Failure> {
        let geom: Handle<dyn GeomSurface> = match kind {
            SurfaceType::BSplineSurface => self.surf.bspline()?.cast(),
            SurfaceType::BezierSurface => self.surf.bezier()?.cast(),
            _ => make_surface(self.surf, true)?,
        };
        GeomAdaptorSurface::with_bounds(&geom, (u.0, u.1, v.0, v.1))
    }

    // Non-analytic surfaces: a local search from (U0, V0) on the patch, then a global
    // one; a result is kept only when orthogonal and nearer than the initial point.
    fn patch_value(&self, kind: SurfaceType, p: Point3, p2d: Point2, surf_pnt_dist: f64) -> Result<Point2, Failure> {
        let [uinf, usup, vinf, vsup] = surface_bounds(self.surf)?;
        let (u0, decal_u) = into_period(p2d.x(), uinf, usup, self.period[0]);
        let (v0, decal_v) = into_period(p2d.y(), vinf, vsup, self.period[1]);
        let little: GeomAdaptorSurface =
            self.little_surface(kind, little_range(u0, uinf, usup), little_range(v0, vinf, vsup))?;
        let shift = |(u, v): (f64, f64)| {
            Point2::new(u - f64::from(decal_u) * self.period[0], v - f64::from(decal_v) * self.period[1])
        };
        let limit: f64 = surf_pnt_dist + precision::square_confusion();
        let mut locext = GenLocateExtPS::new(&little, self.tol_u, self.tol_v);
        locext.perform(p, u0, v0, false)?;
        if locext.is_done() {
            let uv: (f64, f64) = locext.point()?.parameter();
            let dist2: f64 = orthog_sq_value(p, self.surf, uv.0, uv.1)?;
            if dist2 < self.sq_proj_ort_tol && locext.square_distance()? < limit {
                return Ok(shift(uv));
            }
        }
        let ext = ExtPS::with_surface(p, &little, self.tol_u, self.tol_v, ExtFlag::Min, ExtAlgo::Grad)?;
        if ext.is_done() && ext.nb_ext()? >= 1 {
            let good: i32 = nearest(&ext)?;
            let uv: (f64, f64) = ext.point(good)?.parameter();
            let dist2: f64 = orthog_sq_value(p, self.surf, uv.0, uv.1)?;
            if dist2 < self.sq_proj_ort_tol && ext.square_distance(good)? < limit {
                return Ok(shift(uv));
            }
        }
        // Both searches give bad values: the point of the initial 2D curve.
        Ok(p2d)
    }
}

/// The index of the first smallest distance of a done ExtPS.
pub(super) fn nearest(ext: &ExtPS<'_>) -> Result<i32, Failure> {
    let mut dist2_min: f64 = ext.square_distance(1)?;
    let mut good: i32 = 1;
    for i in 2..=ext.nb_ext()? {
        if dist2_min > ext.square_distance(i)? {
            dist2_min = ext.square_distance(i)?;
            good = i;
        }
    }
    Ok(good)
}

/// `ProjLib_PolarFunction`: one 2D point per parameter; D1 is not available.
pub(super) struct PolarFunction<'a> {
    base: FunctionBase,
    pub(super) data: FuncData<'a>,
    failure: RefCell<FailureCapture>,
}

impl<'a> PolarFunction<'a> {
    /// `ProjLib_PolarFunction(C, Surf, InitialCurve2d, Tol3d)`.
    pub(super) fn new(
        curve: &'a dyn Adaptor3dCurve,
        surf: &'a dyn Adaptor3dSurface,
        init_curve2d: &'a dyn Adaptor2dCurve2d,
        tol3d: f64,
    ) -> Result<Self, Failure> {
        let mut base: FunctionBase = FunctionBase::new();
        base.set_number_of_points(0, 1);
        let (u_period, v_period) = compute_periodicity(surf)?;
        let data = FuncData {
            surf,
            curve,
            init_curve2d,
            sq_proj_ort_tol: 10000.0 * tol3d * tol3d,
            tol_u: surf.u_resolution(tol3d)?,
            tol_v: surf.v_resolution(tol3d)?,
            period: [u_period, v_period],
        };
        Ok(Self { base, data, failure: RefCell::new(FailureCapture::default()) })
    }
    /// The first failure a callback kept, after the fit.
    pub(super) fn take_failure(&self) -> Result<(), Failure> {
        self.failure.borrow_mut().take()
    }
    fn keep<T>(&self, value: Result<T, Failure>) -> Option<T> {
        self.failure.borrow_mut().keep(value)
    }
}

impl Function for PolarFunction<'_> {
    fn base(&self) -> &FunctionBase {
        &self.base
    }
    fn first_parameter(&self) -> f64 {
        self.keep(self.data.curve.first_parameter()).unwrap_or(f64::NAN)
    }
    fn last_parameter(&self) -> f64 {
        self.keep(self.data.curve.last_parameter()).unwrap_or(f64::NAN)
    }
    fn value(&self, t: f64, pnt2d: &mut Array1<'_, Point2>, _pnt: &mut Array1<'_, Point3>) -> bool {
        let Some(point) = self.keep(self.data.value(t)) else {
            return false;
        };
        pnt2d[1] = point;
        true
    }
    fn d1(&self, _t: f64, _vec2d: &mut Array1<'_, Vector2>, _vec: &mut Array1<'_, Vector3>) -> bool {
        false
    }
}
