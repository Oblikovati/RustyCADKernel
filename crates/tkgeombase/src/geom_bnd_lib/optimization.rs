// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of GeomBndLib_OptimizationHelpers.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_OptimizationHelpers`: PSO + Brent refinement of a coordinate extremum on
//! a curve (3D and 2D share one generic body, as the native template dispatch does) and
//! PSO + Powell refinement on a surface.
//!
//! The native function objects let an evaluation exception unwind through the
//! optimizer. Here the objects keep the first evaluation failure, report the point as
//! not evaluable, and the helper returns that failure once the optimizer returns.
use super::surface_coord::SurfCoordFunc;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_to_int;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkmath::bvh::{std_max, std_min};
use rustycad_tkmath::math::{Function, Matrix, MultipleVarFunction, Vector};
use rustycad_tkmath::math_opt::{PSOConfig, brent, powell_with_directions, pso};
use rustycad_tkmath::math_utils::{Config, ScalarResult, Status, VectorResult};

/// A curve adaptor whose points expose coordinate `index` (1-based) at a parameter:
/// `Adaptor3d_Curve` (X, Y, Z) and `Adaptor2d_Curve2d` (X, Y).
pub trait CurveCoordinate {
    /// `Value(U).Coord(index)`; `curve.coordinate(0.5, 1)?`.
    fn coordinate(&self, u: f64, index: i32) -> Result<f64, Failure>;
    /// `Resolution(R3d)`; `curve.parametric_resolution(1e-7)?`.
    fn parametric_resolution(&self, tolerance: f64) -> Result<f64, Failure>;
    /// `LastParameter() - FirstParameter()`; `curve.parameter_span()?`.
    fn parameter_span(&self) -> Result<f64, Failure>;
}

impl CurveCoordinate for dyn Adaptor3dCurve + '_ {
    fn coordinate(&self, u: f64, index: i32) -> Result<f64, Failure> {
        self.value(u)?.coord(index)
    }
    fn parametric_resolution(&self, tolerance: f64) -> Result<f64, Failure> {
        self.resolution(tolerance)
    }
    fn parameter_span(&self) -> Result<f64, Failure> {
        Ok(self.last_parameter()? - self.first_parameter()?)
    }
}

impl CurveCoordinate for dyn Adaptor2dCurve2d + '_ {
    fn coordinate(&self, u: f64, index: i32) -> Result<f64, Failure> {
        self.value(u)?.coord(index)
    }
    fn parametric_resolution(&self, tolerance: f64) -> Result<f64, Failure> {
        self.resolution(tolerance)
    }
    fn parameter_span(&self) -> Result<f64, Failure> {
        Ok(self.last_parameter()? - self.first_parameter()?)
    }
}

/// `CurveCoordFunc`/`CurveCoordMVarFunc` and their 2D twins: `sign * Coord(index)` of
/// the curve point, not evaluable outside `[u_min, u_max]`. One object serves as the
/// scalar function of Brent and the one-variable function of PSO.
pub struct CurveCoordFunc<'a, C: ?Sized + CurveCoordinate> {
    curve: &'a C,
    u_min: f64,
    u_max: f64,
    index: i32,
    sign: f64,
    failure: Option<Failure>,
}

impl<'a, C: ?Sized + CurveCoordinate> CurveCoordFunc<'a, C> {
    /// The function of coordinate `index` with `sign` (1 to minimize, -1 to maximize);
    /// `CurveCoordFunc::new(curve, 0.0, 1.0, 1, 1.0)`.
    pub fn new(curve: &'a C, u_min: f64, u_max: f64, index: i32, sign: f64) -> Self {
        Self {
            curve,
            u_min,
            u_max,
            index,
            sign,
            failure: None,
        }
    }
    /// The first evaluation failure, which the native code raises; `func.take_failure()`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take().map_or(Ok(()), Err)
    }
    fn evaluate(&mut self, x: f64, result: &mut f64) -> bool {
        if x < self.u_min || x > self.u_max {
            return false;
        }
        match self.curve.coordinate(x, self.index) {
            Ok(value) => {
                *result = self.sign * value;
                true
            }
            Err(failure) => {
                self.failure.get_or_insert(failure);
                false
            }
        }
    }
}

impl<C: ?Sized + CurveCoordinate> Function for CurveCoordFunc<'_, C> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        self.evaluate(x, result)
    }
}

impl<C: ?Sized + CurveCoordinate> MultipleVarFunction for CurveCoordFunc<'_, C> {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = point.at(0).copied().unwrap_or(f64::NAN);
        self.evaluate(x, value)
    }
}

fn accepted(status: Status) -> bool {
    status == Status::Ok || status == Status::MaxIterations
}

// The Brent configuration of AdjustExtrCurve: relative X tolerance, absolute F tolerance.
fn brent_config(x_tolerance: f64, f_tolerance: f64) -> Config {
    Config {
        x_tolerance,
        f_tolerance,
        max_iterations: 100,
        ..Config::default()
    }
}

/// `AdjustExtrCurve` / `AdjustExtrCurve2d` / `AdjustExtrT`: refine the extremum `extr0`
/// of coordinate `index` over `[u_min, u_max]` by PSO + Brent, or Brent alone on a range
/// below 1% of the curve span; `adjust_extr_curve(curve, 0.0, 0.5, 1.0, 1, 1e-7, true)?`.
pub fn adjust_extr_curve<C: ?Sized + CurveCoordinate>(
    curve: &C,
    u_min: f64,
    u_max: f64,
    extr0: f64,
    index: i32,
    tolerance: f64,
    is_min: bool,
) -> Result<f64, Failure> {
    let sign: f64 = if is_min { 1.0 } else { -1.0 };
    let extr: f64 = sign * extr0;
    let u_tol: f64 = std_max(
        curve.parametric_resolution(tolerance)?,
        precision::p_confusion(),
    );
    let du: f64 = curve.parameter_span()?;
    let rel_tol: f64 = u_tol / std_max(u_min.abs(), u_max.abs());
    let config: Config = brent_config(rel_tol, u_tol);
    if u_max - u_min < 0.01 * du {
        let mut func = CurveCoordFunc::new(curve, u_min, u_max, index, sign);
        let local: ScalarResult = brent(&mut func, u_min, u_max, &config);
        func.take_failure()?;
        if accepted(local.status) {
            return Ok(sign * local.value.unwrap_or(extr));
        }
    }
    let search = CurveSearch {
        curve,
        u_min,
        u_max,
        index,
        sign,
        du,
    };
    search.pso_then_brent(extr, &config)
}

// The PSO + Brent stage of AdjustExtrCurve.
struct CurveSearch<'a, C: ?Sized + CurveCoordinate> {
    curve: &'a C,
    u_min: f64,
    u_max: f64,
    index: i32,
    sign: f64,
    du: f64,
}

impl<C: ?Sized + CurveCoordinate> CurveSearch<'_, C> {
    fn pso_then_brent(&self, extr: f64, config: &Config) -> Result<f64, Failure> {
        let particles: i32 = 8.max(real_to_int(32.0 * (self.u_max - self.u_min) / self.du));
        let swarm: VectorResult = self.run_pso(particles)?;
        let (start, extr): (f64, f64) = self.pso_start(&swarm, extr);
        let step: f64 = std_min(
            0.1 * self.du,
            (self.u_max - self.u_min) / f64::from(particles + 1),
        );
        let mut func =
            CurveCoordFunc::new(self.curve, self.u_min, self.u_max, self.index, self.sign);
        let lower: f64 = std_max(start - step, self.u_min);
        let upper: f64 = std_min(start + step, self.u_max);
        let local: ScalarResult = brent(&mut func, lower, upper, config);
        func.take_failure()?;
        let Some(value) = local.value.filter(|_| accepted(local.status)) else {
            return Ok(self.sign * extr);
        };
        Ok(self.sign * self.keep_pso_minimum(value, &swarm, start))
    }
    fn run_pso(&self, particles: i32) -> Result<VectorResult, Failure> {
        let lower: Vector<'static> = Vector::with_size_filled(1, self.u_min);
        let upper: Vector<'static> = Vector::with_size_filled(1, self.u_max);
        let config: PSOConfig = PSOConfig::new(particles as usize, 100, 1.0e-8);
        let mut func =
            CurveCoordFunc::new(self.curve, self.u_min, self.u_max, self.index, self.sign);
        let swarm: VectorResult = pso(&mut func, &lower, &upper, &config);
        func.take_failure()?;
        Ok(swarm)
    }
    // The Brent seed and the extremum after PSO: the swarm optimum when accepted, the
    // middle of the range otherwise.
    fn pso_start(&self, swarm: &VectorResult, extr: f64) -> (f64, f64) {
        let middle: f64 = (self.u_min + self.u_max) / 2.0;
        if !accepted(swarm.status) {
            return (middle, extr);
        }
        let start: f64 = swarm
            .solution
            .as_ref()
            .and_then(|solution| solution.at(0).ok().copied())
            .unwrap_or(middle);
        (start, swarm.value.unwrap_or(extr))
    }
    // The native `aT.At(0) == aPsoRes.Solution->At(0)` check keeps the PSO minimum when
    // Brent started from the swarm optimum.
    fn keep_pso_minimum(&self, value: f64, swarm: &VectorResult, start: f64) -> f64 {
        let pso_value: Option<f64> = swarm.value;
        let pso_start: Option<f64> = swarm.solution.as_ref().and_then(|s| s.at(0).ok().copied());
        match (pso_value, pso_start) {
            (Some(pso_value), Some(pso_start)) if start == pso_start => std_min(value, pso_value),
            _ => value,
        }
    }
}

/// `AdjustExtrSurf`: refine the extremum `extr0` of coordinate `index` over the patch
/// `[u_min, u_max] x [v_min, v_max]` by PSO + Powell (coordinate directions);
/// `adjust_extr_surf(surface, [0.0, 1.0, 0.0, 1.0], 1.0, 3, 1e-7, false)?`.
pub fn adjust_extr_surf(
    surface: &dyn Adaptor3dSurface,
    patch: [f64; 4],
    extr0: f64,
    index: i32,
    tolerance: f64,
    is_min: bool,
) -> Result<f64, Failure> {
    let [u_min, u_max, v_min, v_max] = patch;
    let sign: f64 = if is_min { 1.0 } else { -1.0 };
    let mut extr: f64 = sign * extr0;
    let mut rel_tol: f64 = 2.0 * tolerance;
    if extr.abs() > tolerance {
        rel_tol /= extr.abs();
    }
    let du: f64 = surface.last_u_parameter()? - surface.first_u_parameter()?;
    let dv: f64 = surface.last_v_parameter()? - surface.first_v_parameter()?;
    let nb_u: i32 = 8.max(real_to_int(32.0 * (u_max - u_min) / du));
    let nb_v: i32 = 8.max(real_to_int(32.0 * (v_max - v_min) / dv));
    let mut func = SurfCoordFunc::new(surface, patch, index, sign)?;
    let swarm: VectorResult = surface_pso(&mut func, patch, nb_u.wrapping_mul(nb_v))?;
    let start: Vector<'static> = surface_start(&swarm, patch)?;
    if accepted(swarm.status) {
        extr = swarm.value.unwrap_or(extr);
    }
    let powell: VectorResult = surface_powell(&mut func, &start, rel_tol, tolerance)?;
    if accepted(powell.status)
        && let Some(value) = powell.value
    {
        extr = std_min(extr, value);
    }
    Ok(sign * extr)
}

fn surface_pso(
    func: &mut SurfCoordFunc<'_>,
    patch: [f64; 4],
    particles: i32,
) -> Result<VectorResult, Failure> {
    let [u_min, u_max, v_min, v_max] = patch;
    let lower: Vector<'static> = pair_vector(u_min, v_min)?;
    let upper: Vector<'static> = pair_vector(u_max, v_max)?;
    let config: PSOConfig = PSOConfig::new(particles as usize, 100, 1.0e-8);
    let swarm: VectorResult = pso(func, &lower, &upper, &config);
    func.take_failure()?;
    Ok(swarm)
}

fn surface_powell(
    func: &mut SurfCoordFunc<'_>,
    start: &Vector<'_>,
    rel_tol: f64,
    tolerance: f64,
) -> Result<VectorResult, Failure> {
    let mut directions: Matrix<'static> = Matrix::with_size_filled(2, 2, 0.0);
    *directions.change_at(0, 0).map_err(array_failure)? = 1.0;
    *directions.change_at(1, 1).map_err(array_failure)? = 1.0;
    let config: Config = Config {
        x_tolerance: rel_tol,
        f_tolerance: tolerance,
        max_iterations: 200,
        ..Config::default()
    };
    let result: VectorResult = powell_with_directions(func, start, &directions, &config);
    func.take_failure()?;
    Ok(result)
}

// The Powell seed: the swarm optimum when PSO was accepted with a solution, the patch
// centre otherwise.
fn surface_start(swarm: &VectorResult, patch: [f64; 4]) -> Result<Vector<'static>, Failure> {
    let [u_min, u_max, v_min, v_max] = patch;
    let centre: Vector<'static> = pair_vector((u_min + u_max) / 2.0, (v_min + v_max) / 2.0)?;
    if !accepted(swarm.status) {
        return Ok(centre);
    }
    let Some(solution) = swarm.solution.as_ref() else {
        return Ok(centre);
    };
    let u: f64 = *solution.at(0)?;
    pair_vector(u, *solution.at(1)?)
}

// A 0-based two-component `math_Vector(size_t{2})`.
pub(super) fn pair_vector(first: f64, second: f64) -> Result<Vector<'static>, Failure> {
    let mut vector: Vector<'static> = Vector::with_size(2);
    *vector.change_at(0)? = first;
    *vector.change_at(1)? = second;
    Ok(vector)
}

fn array_failure(error: impl std::fmt::Display) -> Failure {
    rustycad_tkernel::standard::exceptions::out_of_range(&error.to_string())
}
