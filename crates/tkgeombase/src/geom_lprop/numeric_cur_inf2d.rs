// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomLProp/GeomLProp_NumericCurInf2d.pxx,
// GeomLProp_FuncCurExt2d.pxx and GeomLProp_FuncCurNul2d.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The private numeric search of the curvature extrema and inflections of a 2D
//! curve (`GeomLProp_NumericCurInf2d` with its two root functions).
//!
//! The math root finders take functions that report success as a flag; a curve
//! evaluation that raises is kept in the function, reported as a failed value, and
//! returned as `Err` by the search right after the finder, before any point of that
//! search is stored: the native exception leaves the finder at the same point.
use crate::lprop::CurAndInf;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::{self, Vector2};
use rustycad_tkmath::math::{BracketedRoot, Function, FunctionRoots, FunctionWithDerivative};

// The three derivatives of the curve at `u`.
fn derivatives(curve: &Handle<dyn Geom2dCurve>, u: f64) -> Result<[Vector2; 3], Failure> {
    let (_, d1, d2, d3) = crate::lprop::curve_access::live(curve)?.d3(u)?;
    Ok([d1, d2, d3])
}

// Keep the first evaluation failure of a root function.
fn keep_failure<T>(slot: &mut Option<Failure>, result: Result<T, Failure>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(failure) => {
            slot.get_or_insert(failure);
            None
        }
    }
}

/// `GeomLProp_FuncCurExt2d`: `dKC/dU`, the derivative of the signed curvature
/// `KC = (V1 ^ V2) / |V1|^3`, whose roots are the curvature extrema.
struct FuncCurExt2d<'a> {
    my_curve: &'a Handle<dyn Geom2dCurve>,
    my_eps_x: f64,
    failure: Option<Failure>,
}

impl FuncCurExt2d<'_> {
    // dKC/dU, or None when |V1|^5 is below gp::Resolution().
    fn curvature_derivative(&self, u: f64) -> Result<Option<f64>, Failure> {
        let [d1, d2, d3] = derivatives(self.my_curve, u)?;
        let cross12: f64 = d1.crossed(d2);
        let cross13: f64 = d1.crossed(d3);
        let dot12: f64 = d1.dot(d2);
        let sq_norm1: f64 = d1.square_magnitude();
        let norm1: f64 = sq_norm1.sqrt();
        let norm1_cb: f64 = sq_norm1 * norm1;
        let norm1_pow5: f64 = norm1_cb * sq_norm1;
        if norm1_pow5 < gp::resolution() {
            return Ok(None);
        }
        Ok(Some(
            cross13 / norm1_cb - 3.0 * cross12 * dot12 / norm1_pow5,
        ))
    }

    // `IsMinKC`: whether |KC(u)| exceeds |KC| one step `EpsX` further (backward at
    // the end of the range); false when |V1|^3 vanishes at either point.
    fn is_min_kc(&self, u: f64) -> Result<bool, Failure> {
        let Some(curv_curr) = signed_curvature(self.my_curve, u)? else {
            return Ok(false);
        };
        let mut delta_u: f64 = self.my_eps_x;
        if u + delta_u > crate::lprop::curve_access::live(self.my_curve)?.last_parameter() {
            delta_u = -delta_u;
        }
        let Some(curv_probe) = signed_curvature(self.my_curve, u + delta_u)? else {
            return Ok(false);
        };
        Ok(curv_curr.abs() > curv_probe.abs())
    }
}

// KC = (V1 ^ V2) / |V1|^3, or None when |V1|^3 is below gp::Resolution().
fn signed_curvature(curve: &Handle<dyn Geom2dCurve>, u: f64) -> Result<Option<f64>, Failure> {
    let [d1, d2, _] = derivatives(curve, u)?;
    let cross12: f64 = d1.crossed(d2);
    let sq_norm1: f64 = d1.square_magnitude();
    let norm1_cb: f64 = sq_norm1 * sq_norm1.sqrt();
    if norm1_cb < gp::resolution() {
        return Ok(None);
    }
    Ok(Some(cross12 / norm1_cb))
}

impl Function for FuncCurExt2d<'_> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let value = self.curvature_derivative(x);
        match keep_failure(&mut self.failure, value).flatten() {
            Some(value) => {
                *result = value;
                true
            }
            None => false,
        }
    }
}

impl FunctionWithDerivative for FuncCurExt2d<'_> {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(x, &mut value, derivative)
    }
    // The forward (backward at the end of the range) difference of step EpsX/100.
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let mut delta_u: f64 = self.my_eps_x / 100.0;
        let last = crate::lprop::curve_access::live(self.my_curve).map(|c| c.last_parameter());
        let Some(last) = keep_failure(&mut self.failure, last) else {
            return false;
        };
        if x + delta_u > last {
            delta_u = -delta_u;
        }
        let mut next: f64 = 0.0;
        if !self.value(x, value) || !self.value(x + delta_u, &mut next) {
            return false;
        }
        *derivative = (next - *value) / delta_u;
        true
    }
}

/// `GeomLProp_FuncCurNul2d`: `(V1 ^ V2) / (|V1| |V2|)`, whose roots are the
/// inflections, with its analytic derivative.
struct FuncCurNul2d<'a> {
    my_curve: &'a Handle<dyn Geom2dCurve>,
    failure: Option<Failure>,
}

// F and F' of FuncCurNul2d: (0, 0) when |V2| < 1e-4, None when |V1||V2| is below
// gp::Resolution().
fn normalized_cross(
    curve: &Handle<dyn Geom2dCurve>,
    u: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    let [d1, d2, d3] = derivatives(curve, u)?;
    let cross12: f64 = d1.crossed(d2);
    let cross13: f64 = d1.crossed(d3);
    let dot12: f64 = d1.dot(d2);
    let dot23: f64 = d2.dot(d3);
    let norm1: f64 = d1.magnitude();
    let norm2: f64 = d2.magnitude();
    if norm2 < 1.0e-4 {
        return Ok(Some((0.0, 0.0)));
    }
    if norm1 * norm2 < gp::resolution() {
        return Ok(None);
    }
    let norm_prod: f64 = norm1 * norm2;
    let f: f64 = cross12 / norm_prod;
    let d: f64 = (cross13 - cross12 * dot12 / (norm1 * norm1) - cross12 * dot23 / (norm2 * norm2))
        / norm_prod;
    Ok(Some((f, d)))
}

impl Function for FuncCurNul2d<'_> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let mut derivative: f64 = 0.0;
        self.values(x, result, &mut derivative)
    }
}

impl FunctionWithDerivative for FuncCurNul2d<'_> {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let mut value: f64 = 0.0;
        self.values(x, &mut value, derivative)
    }
    // Both outputs are zeroed first, as natively, whatever the outcome.
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let evaluated = normalized_cross(self.my_curve, x);
        let Some(outcome) = keep_failure(&mut self.failure, evaluated) else {
            return false;
        };
        (*value, *derivative) = (0.0, 0.0);
        let Some((f, d)) = outcome else {
            return false;
        };
        (*value, *derivative) = (f, d);
        true
    }
}

/// `GeomLProp_NumericCurInf2d`: the numeric search of the particular points of a
/// bounded 2D curve on an interval.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct NumericCurInf2d {
    my_is_done: bool,
}

impl NumericCurInf2d {
    /// `PerformCurExt(C, Result)`: the curvature extrema over the curve's range.
    pub(super) fn perform_cur_ext(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        result: &mut CurAndInf,
    ) -> Result<(), Failure> {
        let (first, last) = curve_range(curve)?;
        self.perform_cur_ext_on(curve, first, last, result)
    }

    /// `PerformCurExt(C, UMin, UMax, Result)`: the roots of dKC/dU sampled on 100
    /// points (tolerances 1e-4 of the interval), each refined by a bracketed root
    /// within `EpsH` (tolerance Precision::PConfusion()).
    pub(super) fn perform_cur_ext_on(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        u_min: f64,
        u_max: f64,
        result: &mut CurAndInf,
    ) -> Result<(), Failure> {
        self.my_is_done = true;
        let eps_h: f64 = 1.0e-4 * (u_max - u_min);
        let mut func = FuncCurExt2d {
            my_curve: curve,
            my_eps_x: eps_h,
            failure: None,
        };
        let roots = FunctionRoots::with_options(&mut func, u_min, u_max, 100, [eps_h; 3], 0.0);
        raise_kept(&mut func.failure)?;
        if !roots.is_done() {
            self.my_is_done = false;
            return Ok(());
        }
        for j in 1..=roots.number_of_solutions()? {
            let param: f64 = refined_root(&mut func, roots.value(j)?, eps_h)?;
            result.add_ext_cur(param, func.is_min_kc(param)?);
        }
        Ok(())
    }

    /// `PerformInf(C, Result)`: the inflections over the curve's range.
    pub(super) fn perform_inf(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        result: &mut CurAndInf,
    ) -> Result<(), Failure> {
        let (first, last) = curve_range(curve)?;
        self.perform_inf_on(curve, first, last, result)
    }

    /// `PerformInf(C, UMin, UMax, Result)`: the roots of the normalized cross
    /// product sampled on 30 points with tolerances 1e-6.
    pub(super) fn perform_inf_on(
        &mut self,
        curve: &Handle<dyn Geom2dCurve>,
        u_min: f64,
        u_max: f64,
        result: &mut CurAndInf,
    ) -> Result<(), Failure> {
        self.my_is_done = true;
        let mut func = FuncCurNul2d {
            my_curve: curve,
            failure: None,
        };
        let roots = FunctionRoots::with_options(&mut func, u_min, u_max, 30, [1.0e-6; 3], 0.0);
        raise_kept(&mut func.failure)?;
        if !roots.is_done() {
            self.my_is_done = false;
            return Ok(());
        }
        for j in 1..=roots.number_of_solutions()? {
            result.add_inflection(roots.value(j)?);
        }
        Ok(())
    }

    /// `IsDone()`: whether the last search succeeded.
    pub(super) fn is_done(&self) -> bool {
        self.my_is_done
    }
}

// `theCurve->FirstParameter()` and `LastParameter()`.
fn curve_range(curve: &Handle<dyn Geom2dCurve>) -> Result<(f64, f64), Failure> {
    let object = crate::lprop::curve_access::live(curve)?;
    Ok((object.first_parameter(), object.last_parameter()))
}

// The bracketed refinement of a sampled root on [x - EpsH, x + EpsH]; the sampled
// root when the bracket fails.
fn refined_root(func: &mut FuncCurExt2d<'_>, param: f64, eps_h: f64) -> Result<f64, Failure> {
    let refinement =
        BracketedRoot::new(func, param - eps_h, param + eps_h, precision::p_confusion());
    raise_kept(&mut func.failure)?;
    if refinement.is_done() {
        return refinement.root();
    }
    Ok(param)
}

// Return the evaluation failure a root function kept, if any.
fn raise_kept(slot: &mut Option<Failure>) -> Result<(), Failure> {
    match slot.take() {
        Some(failure) => Err(failure),
        None => Ok(()),
    }
}
