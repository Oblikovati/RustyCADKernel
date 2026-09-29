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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_SameParameter.cxx (Approx_SameParameter_Evaluator, ProjectPointOnCurve, ComputeTolReached, Check, Interpolate, IncreaseNbPoles)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The change of parameter: a cubic B-spline interpolating the sample parameters,
//! its checks, and the evaluator composing it with the pcurve.
use super::{MAX_ARRAY_SIZE, SameParameter, SameParameterData, at, put};
use crate::extrema::LocateExtPC;
use crate::failure_capture::FailureCapture;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use rustycad_tkg3d::adv_approx::AdvApproxEvaluatorFunction;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The degree of the change of parameter.
const DEGREE: i32 = 3;
/// `extrap_mode`: no extrapolation of lower degree.
const EXTRAP_MODE: [i32; 2] = [DEGREE, DEGREE];

/// The change of parameter at `u` and its first `request` derivatives
/// (`BSplCLib::Eval` with flat knots, one dimension).
pub(super) fn change_of_parameter(
    flat_knots: &Array1<'_, f64>,
    poles: &Array1<'_, f64>,
    u: f64,
    request: i32,
) -> Result<[f64; 3], Failure> {
    let mut result: [f64; 3] = [0.0; 3];
    let values: &[f64] = poles.as_slice();
    bsplclib::eval_extrapolated(
        u,
        false,
        request,
        &EXTRAP_MODE,
        DEGREE,
        flat_knots,
        1,
        values,
        &mut result,
    )?;
    Ok(result)
}

/// `Approx_SameParameter_Evaluator`: the pcurve at the changed parameter (its
/// derivative times the parameter derivative). The native evaluator writes nothing
/// for a second derivative and returns 0.
pub(super) struct SameParameterEvaluator<'d> {
    flat_knots: &'d Array1<'static, f64>,
    poles: &'d Array1<'static, f64>,
    hcurve_2d: Handle<dyn Adaptor2dCurve2d>,
    capture: FailureCapture,
}

impl<'d> SameParameterEvaluator<'d> {
    pub(super) fn new(
        flat_knots: &'d Array1<'static, f64>,
        poles: &'d Array1<'static, f64>,
        hcurve_2d: Handle<dyn Adaptor2dCurve2d>,
    ) -> Self {
        Self {
            flat_knots,
            poles,
            hcurve_2d,
            capture: FailureCapture::default(),
        }
    }

    pub(super) fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    fn values(&self, parameter: f64, request: i32, result: &mut [f64]) -> Result<(), Failure> {
        let eval: [f64; 3] = change_of_parameter(self.flat_knots, self.poles, parameter, request)?;
        let curve = self
            .hcurve_2d
            .get()
            .ok_or_else(|| super::null_input("2D curve"))?;
        if request == 0 {
            let p = curve.value(eval[0])?;
            (result[0], result[1]) = (p.x(), p.y());
        } else if request == 1 {
            let (_, v) = curve.d1(eval[0])?;
            let v = v * eval[1];
            (result[0], result[1]) = (v.x(), v.y());
        }
        Ok(())
    }
}

impl AdvApproxEvaluatorFunction for SameParameterEvaluator<'_> {
    fn evaluate(
        &mut self,
        _: i32,
        _: [f64; 2],
        parameter: f64,
        request: i32,
        result: &mut [f64],
    ) -> i32 {
        let evaluated = self.values(parameter, request, result);
        if self.capture.keep(evaluated).is_none() {
            return 1;
        }
        0
    }
}

/// `ProjectPointOnCurve(InitValue, APoint, Tolerance, NumIteration, Curve, Status,
/// Result)`: Newton iterations on `(C(u) - P) . C'(u)` clamped to the curve range;
/// `(Status, Result)`.
pub(super) fn project_point_on_curve(
    init_value: f64,
    point: Point3,
    tolerance: f64,
    num_iteration: i32,
    curve: &dyn Adaptor3dCurve,
) -> Result<(bool, f64), Failure> {
    const DERIVATIVE_TOLERANCE: f64 = 1.0e-12;
    let (mut num_iter, mut param, mut status): (i32, f64, bool) = (0, init_value, false);
    loop {
        num_iter += 1;
        let (a_point, d1, d2): (Point3, Vector3, Vector3) = curve.d2(param)?;
        let vector: Vector3 = Vector3::between(a_point, point);
        let func: f64 = vector.dot(d1);
        let not_done: bool = if func.abs() < tolerance * d1.magnitude() {
            status = true;
            false
        } else {
            let func_derivative: f64 = vector.dot(d2) - d1.dot(d1);
            if func_derivative.abs() > DERIVATIVE_TOLERANCE {
                param -= func / func_derivative;
            }
            param = std_max(param, curve.first_parameter()?);
            param = std_min(param, curve.last_parameter()?);
            true
        };
        if !(not_done && num_iter <= num_iteration) {
            return Ok((status, param));
        }
    }
}

fn std_max(a: f64, b: f64) -> f64 {
    if a < b { b } else { a }
}

fn std_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// `ComputeTolReached(c3d, cons, nbp)`: 1.05 times the largest distance at `nbp + 1`
/// parameters (infinite when an evaluation raises or is infinite), at least
/// Precision::Confusion().
pub(super) fn compute_tol_reached(
    c3d: &dyn Adaptor3dCurve,
    cons: &Adaptor3dCurveOnSurface,
    nbp: i32,
) -> Result<f64, Failure> {
    let (first, last) = (c3d.first_parameter()?, c3d.last_parameter()?);
    let mut d2: f64 = 0.0;
    for i in 0..=nbp {
        let t: f64 = f64::from(i) / f64::from(nbp);
        let u: f64 = first * (1.0 - t) + last * t;
        // The native try/catch turns any exception into an infinite deviation.
        let (Ok(pc3d), Ok(pcons)) = (c3d.value(u), cons.value(u)) else {
            d2 = precision::infinite();
            break;
        };
        if [pcons.x(), pcons.y(), pcons.z()]
            .into_iter()
            .any(precision::is_infinite)
        {
            d2 = precision::infinite();
            break;
        }
        d2 = std_max(d2, pc3d.square_distance(&pcons));
    }
    let deviation: f64 = (1. + 0.05) * d2.sqrt();
    Ok(std_max(deviation, precision::confusion()))
}

/// The inputs of Check.
pub(super) struct CheckInput<'a> {
    pub(super) flat_knots: &'a Array1<'static, f64>,
    pub(super) poles: &'a Array1<'static, f64>,
    pub(super) nbp: i32,
    pub(super) pc3d: &'a [f64],
    pub(super) c3d: &'a dyn Adaptor3dCurve,
    pub(super) cons: &'a Adaptor3dCurveOnSurface,
}

/// `Check(FlatKnots, Poles, nbp, pc3d, c3d, cons, tol, oldtol)`: the change of
/// parameter must stay increasing within `[3 p0 - 2 pn, 3 pn - 2 p0]` (bounded by the
/// pcurve range, the OCC5898 criterion); `tol` receives the largest deviation at
/// `2 nbp + 1` points (infinite when the parameters fail), and the result is whether
/// the poles are ordered and the deviation is within the tolerance or not much
/// better than `oldtol`.
pub(super) fn check(input: &CheckInput<'_>, tol: &mut f64, oldtol: f64) -> Result<bool, Failure> {
    let (p0, pn) = (at(input.pc3d, 0)?, at(input.pc3d, input.nbp - 1)?);
    let mut param_first: f64 = 3.0 * p0 - 2.0 * pn;
    let mut param_last: f64 = 3.0 * pn - 2.0 * p0;
    let (first_par, last_par) = (input.cons.first_parameter()?, input.cons.last_parameter()?);
    if param_first < first_par {
        param_first = first_par;
    }
    if param_last > last_par {
        param_last = last_par;
    }
    let (d, nn) = (*tol, 2 * input.nbp);
    let unsurnn: f64 = 1.0 / f64::from(nn);
    let (mut d2, mut tprev): (f64, f64) = (0.0, param_first);
    for i in 0..=nn {
        let t: f64 = unsurnn * f64::from(i);
        let tc3d: f64 = p0 * (1.0 - t) + pn * t;
        let pc3d: Point3 = input.c3d.value(tc3d)?;
        let tcons: f64 = change_of_parameter(input.flat_knots, input.poles, tc3d, 0)?[0];
        if tcons < tprev || tcons > param_last {
            *tol = precision::infinite();
            return Ok(false);
        }
        tprev = tcons;
        let temp: f64 = pc3d.square_distance(&input.cons.value(tcons)?);
        if temp > d2 {
            d2 = temp;
        }
    }
    *tol = d2.sqrt();
    let poles: &[f64] = input.poles.as_slice();
    if poles.windows(2).any(|pair| pair[0] > pair[1]) {
        return Ok(false);
    }
    Ok(*tol <= d || *tol > 0.8 * oldtol)
}

impl SameParameter {
    /// `Interpolate(theData, aTangFirst, aTangLast, thePoles, theFlatKnots)`: the
    /// cubic B-spline through the sample parameters with the end tangents; the
    /// native BSplCLib::Interpolate raises Standard_OutOfRange instead of reporting a
    /// singular system, so the result is always true when it returns.
    pub(super) fn interpolate(
        data: &SameParameterData,
        [tang_first, tang_last]: [f64; 2],
        poles: &mut Array1<'static, f64>,
        flat_knots: &mut Array1<'static, f64>,
    ) -> Result<bool, Failure> {
        let num_poles: i32 = data.nb_pnt + 3;
        let failure = |e| array_failure(e, "Approx_SameParameter::Interpolate");
        let mut contact: Array1<'static, i32> =
            Array1::with_bounds(1, num_poles).map_err(failure)?;
        let mut params: Array1<'static, f64> =
            Array1::with_bounds(1, num_poles).map_err(failure)?;
        contact.init(0);
        contact.set_value(2, 1).map_err(failure)?;
        contact.set_value(num_poles - 1, 1).map_err(failure)?;
        for k in 1..=4 {
            flat_knots.set_value(k, data.c3d_pf).map_err(failure)?;
            flat_knots
                .set_value(num_poles + k, data.c3d_pl)
                .map_err(failure)?;
        }
        poles.set_value(1, data.c2d_pf).map_err(failure)?;
        poles.set_value(num_poles, data.c2d_pl).map_err(failure)?;
        poles.set_value(2, tang_first).map_err(failure)?;
        poles.set_value(num_poles - 1, tang_last).map_err(failure)?;
        for k in [1, 2] {
            params.set_value(k, data.c3d_pf).map_err(failure)?;
        }
        for k in [num_poles - 1, num_poles] {
            params.set_value(k, data.c3d_pl).map_err(failure)?;
        }
        for ii in 3..=num_poles - 2 {
            poles
                .set_value(ii, at(&data.pc2d, ii - 2)?)
                .map_err(failure)?;
            let p3: f64 = at(&data.pc3d, ii - 2)?;
            flat_knots.set_value(ii + 2, p3).map_err(failure)?;
            params.set_value(ii, p3).map_err(failure)?;
        }
        let mut inversion_problem: i32 = 0;
        bsplclib::interpolate_flat(
            DEGREE,
            flat_knots,
            &params,
            &contact,
            1,
            poles.as_mut_slice(),
            &mut inversion_problem,
        )?;
        Ok(inversion_problem == 0)
    }

    /// `IncreaseNbPoles(thePoles, theFlatKnots, theData, theBestSqTol)`: a projected
    /// midpoint in each interval where the change of parameter leaves the pcurve
    /// parameters, else a projected midpoint in every interval; `best_sq_tol` grows
    /// with the projection distances; false when no point was added.
    pub(super) fn increase_nb_poles(
        &self,
        poles: &Array1<'static, f64>,
        flat_knots: &Array1<'static, f64>,
        data: &mut SameParameterData,
        best_sq_tol: &mut f64,
    ) -> Result<bool, Failure> {
        let c3d: &dyn Adaptor3dCurve = self.c3d()?;
        let mut projector: LocateExtPC<'_> = LocateExtPC::new();
        let range = (c3d.first_parameter()?, c3d.last_parameter()?);
        projector.initialize(c3d, range, data.tol)?;
        let mut adder = MidpointAdder {
            same: self,
            projector,
            curp: 0.0,
            projok: false,
        };
        let newcount: i32 = adder.add_midpoints(data, best_sq_tol, Some((poles, flat_knots)))?;
        if data.nb_pnt != newcount && newcount < MAX_ARRAY_SIZE - 1 {
            data.swap(newcount);
            return Ok(true);
        }
        let newcount: i32 = adder.add_midpoints(data, best_sq_tol, None)?;
        if data.nb_pnt != newcount {
            data.swap(newcount);
            return Ok(true);
        }
        Ok(false)
    }
}

/// The projector and the last projection of IncreaseNbPoles, kept across both
/// passes as the native locals are.
struct MidpointAdder<'s> {
    same: &'s SameParameter,
    projector: LocateExtPC<'s>,
    curp: f64,
    projok: bool,
}

impl MidpointAdder<'_> {
    // One pass: the samples copied to the new arrays with a projected midpoint after
    // each (only where the change of parameter leaves `[pc2d(ii), pc2d(ii+1)]` when
    // `change` is given); returns the new count.
    fn add_midpoints(
        &mut self,
        data: &mut SameParameterData,
        best_sq_tol: &mut f64,
        change: Option<(&Array1<'static, f64>, &Array1<'static, f64>)>,
    ) -> Result<i32, Failure> {
        let mut newcount: i32 = 0;
        for ii in 0..data.nb_pnt {
            let (p2, p3) = (at(&data.pc2d, ii)?, at(&data.pc3d, ii)?);
            put(&mut data.new_pc2d, newcount, p2)?;
            put(&mut data.new_pc3d, newcount, p3)?;
            newcount += 1;
            if data.nb_pnt - ii + newcount == MAX_ARRAY_SIZE {
                continue;
            }
            let (next2, next3) = (at(&data.pc2d, ii + 1)?, at(&data.pc3d, ii + 1)?);
            if let Some((poles, flat_knots)) = change {
                let eval: f64 = change_of_parameter(flat_knots, poles, 0.5 * (p3 + next3), 0)?[0];
                if !(eval < p2 || eval > next2) {
                    continue;
                }
            }
            let ucons: f64 = 0.5 * (p2 + next2);
            let uc3d: f64 = 0.5 * (p3 + next3);
            self.project(data, (ucons, uc3d), best_sq_tol)?;
            if self.projok
                && self.curp > p3 + self.same.delta_min
                && self.curp < next3 - self.same.delta_min
            {
                put(&mut data.new_pc3d, newcount, self.curp)?;
                put(&mut data.new_pc2d, newcount, ucons)?;
                newcount += 1;
            }
        }
        let (last2, last3) = (at(&data.pc2d, data.nb_pnt)?, at(&data.pc3d, data.nb_pnt)?);
        put(&mut data.new_pc3d, newcount, last3)?;
        put(&mut data.new_pc2d, newcount, last2)?;
        Ok(newcount)
    }

    // The projection of the pcurve point at `ucons` from `uc3d`: the local extremum
    // (raising the best squared tolerance to its distance), else the Newton
    // iterations, which set both `projok` and `curp`.
    fn project(
        &mut self,
        data: &SameParameterData,
        (ucons, uc3d): (f64, f64),
        best_sq_tol: &mut f64,
    ) -> Result<(), Failure> {
        let pcons: Point3 = data.cons.value(ucons)?;
        self.projector.perform(pcons, uc3d)?;
        if self.projector.is_done() {
            self.curp = self.projector.point()?.parameter();
            let dist_2: f64 = self.projector.square_distance()?;
            if dist_2 > *best_sq_tol {
                *best_sq_tol = dist_2;
            }
            self.projok = true;
            return Ok(());
        }
        (self.projok, self.curp) =
            project_point_on_curve(uc3d, pcons, data.tol, 30, self.same.c3d()?)?;
        Ok(())
    }
}
