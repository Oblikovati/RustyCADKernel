// Created on: 1995-07-18
// Created by: Modelistation
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
// Rust translation of Extrema_GenExtCS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GenExtCS`: the minimum of the distance between a curve and a surface: a
//! particle swarm (math_PSO) seeded from samples, over `(t, u, v)`, over `(u, v)`
//! for a conic or over `t` for a quadric, refined by math_FunctionSetRoot on
//! [`FuncExtCS`] on each half of a closed curve.
mod conic;
mod glob_min;

use super::func_ext_cs::FuncExtCS;
use crate::extrema::failures::{extremum_index, not_done, null_object};
use crate::extrema::pon::{POnCurv, POnSurf};
use crate::native_failure::array_failure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomHyperbola, GeomTrimmedCurve};
use rustycad_tkg3d::geom_grid_eval::GeomGridEvalSurface;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::{FunctionSetRoot, Vector};

/// `MaxParamVal`: the bound replacing an infinite parameter.
const MAX_PARAM_VAL: f64 = 1.0e+10;
/// `aBorderDivisor`: the samples stay a ten-thousandth of the range inside it.
const BORDER_DIVISOR: f64 = 1.0e+4;
/// `HyperbolaLimit`: `ln(MaxParamVal)`, the bound on a hyperbola parameter.
const HYPERBOLA_LIMIT: f64 = 23.;
/// The number of particles of math_PSO.
const NB_PARTICLES: i32 = 48;

// `IsQuadric`.
fn is_quadric(surface_type: SurfaceType) -> bool {
    matches!(
        surface_type,
        SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus
    )
}

// `IsConic`.
fn is_conic(curve_type: CurveType) -> bool {
    matches!(
        curve_type,
        CurveType::Line
            | CurveType::Circle
            | CurveType::Ellipse
            | CurveType::Hyperbola
            | CurveType::Parabola
    )
}

// `GetCurvMaxParamVal`: the parameter bound of a curve (of a hyperbola, or an offset
// of a possibly trimmed hyperbola, to avoid FPE).
fn get_curv_max_param_val(curve: &dyn Adaptor3dCurve) -> Result<f64, Failure> {
    if curve.get_type()? == CurveType::Hyperbola {
        return Ok(HYPERBOLA_LIMIT);
    }
    if curve.get_type()? == CurveType::OffsetCurve {
        let mut basis = curve.offset_curve()?.basis_curve();
        let trimmed: Handle<GeomTrimmedCurve> = Handle::down_cast(&basis);
        if let Some(trimmed) = trimmed.get() {
            basis = trimmed.basis_curve();
        }
        if Handle::<GeomHyperbola>::down_cast(&basis).is_some() {
            return Ok(HYPERBOLA_LIMIT);
        }
    }
    Ok(MAX_PARAM_VAL)
}

// `GetSurfMaxParamVals`: the U and V bounds of a surface (from a swept hyperbola).
fn get_surf_max_param_vals(surface: &dyn Adaptor3dSurface) -> Result<(f64, f64), Failure> {
    match surface.get_type()? {
        SurfaceType::SurfaceOfExtrusion => Ok((
            get_curv_max_param_val(&*surface.basis_curve()?)?,
            MAX_PARAM_VAL,
        )),
        SurfaceType::SurfaceOfRevolution => Ok((
            MAX_PARAM_VAL,
            get_curv_max_param_val(&*surface.basis_curve()?)?,
        )),
        SurfaceType::OffsetSurface => get_surf_max_param_vals(&*surface.basis_surface()?),
        _ => Ok((MAX_PARAM_VAL, MAX_PARAM_VAL)),
    }
}

/// `Extrema_GenExtCS`; `GenExtCS::with_curve(&curve, &surface, (12, 10, 10), 1e-9, 1e-9)?`.
#[derive(Default)]
pub struct GenExtCS<'a> {
    done: bool,
    tmin: f64,
    tsup: f64,
    umin: f64,
    usup: f64,
    vmin: f64,
    vsup: f64,
    tsample: i32,
    usample: i32,
    vsample: i32,
    tol1: f64,
    tol2: f64,
    f: FuncExtCS<'a>,
    s: Option<&'a dyn Adaptor3dSurface>,
    surf_pnts: Vec<Vec<Point3>>,
}

impl<'a> GenExtCS<'a> {
    /// `Extrema_GenExtCS()`; `GenExtCS::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_GenExtCS(C, S, NbT, NbU, NbV, Tol1, Tol2)`: on the curve and surface
    /// ranges; `GenExtCS::with_curve(&curve, &surface, (12, 10, 10), 1e-9, 1e-9)?`.
    pub fn with_curve(
        curve: &'a dyn Adaptor3dCurve,
        surface: &'a dyn Adaptor3dSurface,
        (nb_t, nb_u, nb_v): (i32, i32, i32),
        tol1: f64,
        tol2: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(surface, nb_u, nb_v, tol2)?;
        extrema.perform(curve, nb_t, tol1)?;
        Ok(extrema)
    }
    /// `Extrema_GenExtCS(C, S, NbT, NbU, NbV, tmin, tsup, Umin, Usup, Vmin, Vsup, Tol1,
    /// Tol2)`; `GenExtCS::with_ranges(&curve, &surface, (12, 10, 10), [0., 1., 0., 1.,
    /// 0., 1.], (1e-9, 1e-9))?`.
    pub fn with_ranges(
        curve: &'a dyn Adaptor3dCurve,
        surface: &'a dyn Adaptor3dSurface,
        (nb_t, nb_u, nb_v): (i32, i32, i32),
        [tmin, tsup, umin, usup, vmin, vsup]: [f64; 6],
        (tol1, tol2): (f64, f64),
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize_range(surface, (nb_u, nb_v), (umin, usup), (vmin, vsup), tol2)?;
        extrema.perform_range(curve, nb_t, (tmin, tsup), tol1)?;
        Ok(extrema)
    }
    /// `Initialize(S, NbU, NbV, Tol2)`: on the surface bounds;
    /// `extrema.initialize(&surface, 10, 10, 1e-9)?`.
    pub fn initialize(
        &mut self,
        surface: &'a dyn Adaptor3dSurface,
        nb_u: i32,
        nb_v: i32,
        tol2: f64,
    ) -> Result<(), Failure> {
        let u = (surface.first_u_parameter()?, surface.last_u_parameter()?);
        let v = (surface.first_v_parameter()?, surface.last_v_parameter()?);
        (self.umin, self.usup, self.vmin, self.vsup) = (u.0, u.1, v.0, v.1);
        self.initialize_range(surface, (nb_u, nb_v), u, v, tol2)
    }
    /// `Initialize(S, NbU, NbV, Umin, Usup, Vmin, Vsup, Tol2)`: the bounds (infinite
    /// ones replaced by `±1e10`, or 23 on a swept hyperbola) and the surface points of
    /// the `NbU + 1` by `NbV + 1` sampling grid;
    /// `extrema.initialize_range(&surface, (10, 10), (0., 1.), (0., 1.), 1e-9)?`.
    pub fn initialize_range(
        &mut self,
        surface: &'a dyn Adaptor3dSurface,
        (nb_u, nb_v): (i32, i32),
        (umin, usup): (f64, f64),
        (vmin, vsup): (f64, f64),
        tol2: f64,
    ) -> Result<(), Failure> {
        self.s = Some(surface);
        (self.usample, self.vsample) = (nb_u, nb_v);
        (self.umin, self.usup, self.vmin, self.vsup) = (umin, usup, vmin, vsup);
        self.tol2 = tol2;
        let (umaxpar, vmaxpar) = get_surf_max_param_vals(surface)?;
        if precision::is_infinite(self.usup) {
            self.usup = umaxpar;
        }
        if precision::is_infinite(self.umin) {
            self.umin = -umaxpar;
        }
        if precision::is_infinite(self.vsup) {
            self.vsup = vmaxpar;
        }
        if precision::is_infinite(self.vmin) {
            self.vmin = -vmaxpar;
        }
        self.sample_surface(surface)
    }

    // The surface points of the sampling grid, a ten-thousandth inside the bounds.
    fn sample_surface(&mut self, surface: &dyn Adaptor3dSurface) -> Result<(), Failure> {
        let du: f64 = (self.usup - self.umin) / BORDER_DIVISOR;
        let dv: f64 = (self.vsup - self.vmin) / BORDER_DIVISOR;
        let (min_u, min_v) = (self.umin + du, self.vmin + dv);
        let (max_u, max_v) = (self.usup - du, self.vsup - dv);
        let u_params = accumulated(
            min_u,
            (max_u - min_u) / f64::from(self.usample),
            self.usample,
        )?;
        let v_params = accumulated(
            min_v,
            (max_v - min_v) / f64::from(self.vsample),
            self.vsample,
        )?;
        let grid =
            GeomGridEvalSurface::from_adaptor(surface)?.evaluate_grid(&u_params, &v_params)?;
        self.surf_pnts = Vec::with_capacity((self.usample + 1).max(0) as usize);
        for i in 0..=self.usample {
            let mut row: Vec<Point3> = Vec::with_capacity((self.vsample + 1).max(0) as usize);
            for j in 0..=self.vsample {
                row.push(
                    *grid
                        .value(i + 1, j + 1)
                        .map_err(|error| array_failure(error, "NCollection_Array2::Value"))?,
                );
            }
            self.surf_pnts.push(row);
        }
        Ok(())
    }

    fn surface(&self) -> Result<&'a dyn Adaptor3dSurface, Failure> {
        self.s
            .ok_or_else(|| null_object("Extrema_GenExtCS::Perform()"))
    }
    /// `Perform(C, NbT, Tol1)`: on the curve range; `extrema.perform(&curve, 12, 1e-9)?`.
    pub fn perform(
        &mut self,
        curve: &'a dyn Adaptor3dCurve,
        nb_t: i32,
        tol1: f64,
    ) -> Result<(), Failure> {
        (self.tmin, self.tsup) = (curve.first_parameter()?, curve.last_parameter()?);
        self.perform_range(curve, nb_t, (self.tmin, self.tsup), tol1)
    }
    /// `Perform(C, NbT, tmin, tsup, Tol1)`: the global search and the root refinement
    /// on each half of a closed curve covering more than two thirds of its period,
    /// then only the minima of a divided curve;
    /// `extrema.perform_range(&curve, 12, (0., 1.), 1e-9)?`.
    pub fn perform_range(
        &mut self,
        curve: &'a dyn Adaptor3dCurve,
        nb_t: i32,
        (tmin, tsup): (f64, f64),
        tol1: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        let surface = self.surface()?;
        self.f.initialize(curve, surface);
        (self.tmin, self.tsup, self.tol1, self.tsample) = (tmin, tsup, tol1, nb_t);
        let c_max_val: f64 = get_curv_max_param_val(curve)?;
        if precision::is_infinite(self.tsup) {
            self.tsup = c_max_val;
        }
        if precision::is_infinite(self.tmin) {
            self.tmin = -c_max_val;
        }
        let nb_var: i32 = if is_quadric(surface.get_type()?) {
            1
        } else if is_conic(curve.get_type()?) {
            2
        } else {
            3
        };
        let tol: Vector<'static> = triple(self.tol1, self.tol2, self.tol2)?;
        let mut nb_int_c: i32 = 1;
        if curve.is_closed()? || curve.is_periodic()? {
            let period: f64 = curve.period()?;
            if curve.last_parameter()? - curve.first_parameter()? > 2. * period / 3. {
                nb_int_c = 2;
            }
        }
        let dt: f64 = (self.tsup - self.tmin) / f64::from(nb_int_c);
        for an_int in 1..=nb_int_c {
            let t_inf: f64 = self.tmin + f64::from(an_int - 1) * dt;
            let tuv_inf: Vector<'static> = triple(t_inf, self.umin, self.vmin)?;
            let tuv_sup: Vector<'static> = triple(t_inf + dt, self.usup, self.vsup)?;
            let mut tuv: Vector<'static> = Vector::with_bounds(1, 3)?;
            match nb_var {
                3 => self.glob_min_gen_cs(curve, (&tuv_inf, &tuv_sup), &mut tuv)?,
                2 => self.glob_min_conic_s(curve, (&tuv_inf, &tuv_sup), &mut tuv)?,
                _ => self.glob_min_c_quadric(curve, (&tuv_inf, &tuv_sup), &mut tuv)?,
            }
            let mut solver = FunctionSetRoot::new(&self.f, &tol, 100)?;
            solver.perform_bounded(&mut self.f, &tuv, &tuv_inf, &tuv_sup, false)?;
            self.f.take_failure()?;
        }
        if nb_int_c > 1 && self.f.nb_ext() > 1 {
            self.keep_minima();
        }
        self.done = true;
        Ok(())
    }

    // Removes the "false" extrema caused by dividing the curve interval: only the
    // solutions within SquareConfusion of the minimum remain.
    fn keep_minima(&mut self) {
        let sq_dists: Vec<f64> = std::mem::take(self.f.square_distances());
        let on_curve: Vec<POnCurv> = std::mem::take(self.f.points_on_curve());
        let on_surf: Vec<POnSurf> = std::mem::take(self.f.points_on_surf());
        let mut min_dist: f64 = sq_dists[0];
        for &dist in &sq_dists[1..] {
            if dist < min_dist {
                min_dist = dist;
            }
        }
        let tol: f64 = precision::square_confusion();
        for i in 0..sq_dists.len() {
            if (sq_dists[i] - min_dist).abs() <= tol {
                self.f.square_distances().push(sq_dists[i]);
                self.f.points_on_curve().push(on_curve[i]);
                self.f.points_on_surf().push(on_surf[i]);
            }
        }
    }
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GenExtCS::NbExt()"));
        }
        Ok(self.f.nb_ext())
    }
    /// `SquareDistance(N)`; `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        extremum_index("Extrema_GenExtCS::SquareDistance()", n, || self.nb_ext())?;
        self.f.square_distance(n)
    }
    /// `PointOnCurve(N)`; `extrema.point_on_curve(1)?`.
    pub fn point_on_curve(&self, n: i32) -> Result<POnCurv, Failure> {
        extremum_index("Extrema_GenExtCS::PointOnCurve()", n, || self.nb_ext())?;
        self.f.point_on_curve(n)
    }
    /// `PointOnSurface(N)`; `extrema.point_on_surface(1)?`.
    pub fn point_on_surface(&self, n: i32) -> Result<POnSurf, Failure> {
        extremum_index("Extrema_GenExtCS::PointOnSurface()", n, || self.nb_ext())?;
        self.f.point_on_surface(n)
    }
}

/// A 1-based math_Vector of three values.
pub(super) fn triple(a: f64, b: f64, c: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_bounds(1, 3)?;
    (vector[1], vector[2], vector[3]) = (a, b, c);
    Ok(vector)
}

/// The `n + 1` parameters `start + i step`, accumulated as natively, as a 1-based
/// array for GeomGridEval.
fn accumulated(
    start: f64,
    step: f64,
    n: i32,
) -> Result<rustycad_tkernel::ncollection::Array1<'static, f64>, Failure> {
    let mut array = rustycad_tkernel::ncollection::Array1::with_bounds(1, n + 1)
        .map_err(|error| array_failure(error, "Extrema_GenExtCS parameters"))?;
    let mut value: f64 = start;
    for index in 1..=n + 1 {
        array[index] = value;
        value += step;
    }
    Ok(array)
}
