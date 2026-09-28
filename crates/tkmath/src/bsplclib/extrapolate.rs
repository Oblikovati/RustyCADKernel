// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (Eval with ExtrapMode) and BSplCLib_CurveComputation.pxx (BSplCLib_Eval) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation from flat knots through the basis functions, with optional Taylor
//! extrapolation of lower degree outside the knot range.
//!
//! In the extrapolation branch OCCT expands a Taylor polynomial of `LocalRequest+1`
//! terms into a buffer of that size but evaluates it as a polynomial of `Degree`,
//! reading uninitialized stack memory for the missing terms. The port reads zeros
//! there, which evaluates the intended Taylor polynomial; values the native code
//! reads from initialized parts of the buffer (stale pole terms read as weight
//! terms in the rational variant) are reproduced.
use super::basis::eval_bspline_basis;
use super::{MAX_DEGREE, flat_points};
use crate::math::Matrix;
use crate::plib::PolePoint;
use crate::plib::access::{array_value, non_negative, require_length};
use crate::plib::eval_polynomial;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const EVAL: &str = "BSplCLib::Eval";

/// Evaluates `derivative_request` derivatives at `parameter` from flat knots
/// (`BSplCLib::Eval(U, PeriodicFlag, DerivativeRequest, ExtrapMode, Degree, FlatKnots,
/// ArrayDimension, Poles, Result)`).
///
/// `extrap_mode[0]`/`[1]` is the degree used before the first/after the last knot when
/// it is below `degree`. A basis evaluation failure leaves `results` untouched, as natively;
/// `bsplclib::eval_extrapolated(0.5, false, 1, &[3, 3], 3, &flat, 1, &poles, &mut out)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native nine-argument signature.
pub fn eval_extrapolated(
    parameter: f64,
    periodic_flag: bool,
    derivative_request: i32,
    extrap_mode: &[i32; 2],
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    array_dimension: i32,
    poles: &[f64],
    results: &mut [f64],
) -> Result<(), Failure> {
    let setup = EvalSetup::new(
        parameter,
        periodic_flag,
        derivative_request,
        extrap_mode,
        degree,
        flat_knots,
        array_dimension,
    )?;
    let Some(basis) = setup.basis(flat_knots)? else {
        return Ok(());
    };
    let dim: usize = setup.dimension;
    if !setup.extrapolating {
        require_length(
            EVAL,
            "Results",
            results.len(),
            (setup.local_request + 1) * dim,
        )?;
        for ii in 0..=setup.local_request {
            let row = setup.combine(&basis, ii, poles, None)?;
            results[ii * dim..(ii + 1) * dim].copy_from_slice(&row);
        }
        return Ok(());
    }
    let taylor: Vec<f64> = setup.taylor_poles(&basis, poles, None)?;
    setup.evaluate_taylor(&taylor, dim, results)
}

/// Rational variant giving the homogeneous numerator in `poles_results` and the
/// denominator in `weights_results` (`BSplCLib::Eval` with Weights).
///
/// Quirk kept: for derivative rows after the first, the native code clears
/// `WeightsResults[row*ArrayDimension]` and accumulates into `WeightsResults[row]`
/// without clearing it, so those weight derivatives add to the values passed in;
/// a cleared position beyond `weights_results` is rejected;
/// `bsplclib::eval_extrapolated_rational(0.5, false, 0, &[3, 3], 3, &flat, 1, &p, &w, &mut pr, &mut wr)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native eleven-argument signature.
pub fn eval_extrapolated_rational(
    parameter: f64,
    periodic_flag: bool,
    derivative_request: i32,
    extrap_mode: &[i32; 2],
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    array_dimension: i32,
    poles: &[f64],
    weights: &[f64],
    poles_results: &mut [f64],
    weights_results: &mut [f64],
) -> Result<(), Failure> {
    let setup = EvalSetup::new(
        parameter,
        periodic_flag,
        derivative_request,
        extrap_mode,
        degree,
        flat_knots,
        array_dimension,
    )?;
    let Some(basis) = setup.basis(flat_knots)? else {
        return Ok(());
    };
    let dim: usize = setup.dimension;
    if !setup.extrapolating {
        require_length(
            EVAL,
            "PolesResults",
            poles_results.len(),
            (setup.local_request + 1) * dim,
        )?;
        for ii in 0..=setup.local_request {
            let row = setup.combine(&basis, ii, poles, Some(weights))?;
            poles_results[ii * dim..(ii + 1) * dim].copy_from_slice(&row);
            let cleared: usize = ii * dim;
            require_length(
                EVAL,
                "WeightsResults",
                weights_results.len(),
                cleared.max(ii) + 1,
            )?;
            weights_results[cleared] = 0.0e0;
            for jj in 0..setup.order {
                let (pole, term) = setup.basis_term(&basis, ii, jj)?;
                weights_results[ii] += buffer(weights, pole)? * term;
            }
        }
        return Ok(());
    }
    let mut taylor: Vec<f64> = setup.taylor_poles(&basis, poles, Some(weights))?;
    setup.evaluate_taylor(&taylor, dim, poles_results)?;
    setup.taylor_weights(&basis, weights, &mut taylor)?;
    setup.evaluate_taylor(&taylor, 1, weights_results)
}

struct EvalSetup {
    degree: i32,
    order: usize,
    dimension: usize,
    derivative_request: i32,
    local_request: usize,
    local_parameter: f64,
    delta: f64,
    extrapolating: bool,
    modulus: i32,
}

impl EvalSetup {
    fn new(
        parameter: f64,
        periodic_flag: bool,
        derivative_request: i32,
        extrap_mode: &[i32; 2],
        degree: i32,
        flat_knots: &Array1<'_, f64>,
        array_dimension: i32,
    ) -> Result<Self, Failure> {
        non_negative(EVAL, "DerivativeRequest", derivative_request)?;
        let dimension: usize = non_negative(EVAL, "ArrayDimension", array_dimension)?;
        let first: f64 = array_value(flat_knots, 2)?;
        let last: f64 = array_value(flat_knots, flat_knots.upper() - 1)?;
        let mut local_parameter: f64 = parameter;
        if periodic_flag {
            local_parameter = periodic_parameter(parameter, first, last)?;
        }
        let mut local_request: i32 = derivative_request;
        let mut extrapolating: bool = false;
        if parameter < first && local_request < extrap_mode[0] && extrap_mode[0] < degree {
            local_request = extrap_mode[0];
            local_parameter = first;
            extrapolating = true;
        }
        if parameter > last && local_request < extrap_mode[1] && extrap_mode[1] < degree {
            local_request = extrap_mode[1];
            local_parameter = last;
            extrapolating = true;
        }
        if local_request > degree {
            local_request = degree;
        }
        local_matrix_check(local_request, degree + 1)?;
        let length: i32 = flat_knots.len() as i32;
        Ok(Self {
            degree,
            order: non_negative(EVAL, "Degree", degree)? + 1,
            dimension,
            derivative_request,
            local_request: local_request as usize,
            local_parameter,
            delta: parameter - local_parameter,
            extrapolating,
            modulus: if periodic_flag {
                length - degree - 1
            } else {
                length - degree
            },
        })
    }

    // The basis matrix (LocalRequest+1 x Order), or None when EvalBsplineBasis fails.
    fn basis(
        &self,
        flat_knots: &Array1<'_, f64>,
    ) -> Result<Option<(Matrix<'static>, i32)>, Failure> {
        let rows: i32 = self.local_request as i32 + 1;
        let mut basis: Matrix<'static> = Matrix::filled(1, rows, 1, self.order as i32, 0.0)
            .map_err(|error| Failure::out_of_range(&format!("{EVAL}: {error}")))?;
        let mut first: i32 = 0;
        let status: i32 = eval_bspline_basis(
            self.local_request as i32,
            self.order as i32,
            flat_knots,
            self.local_parameter,
            &mut first,
            &mut basis,
            false,
        )?;
        if status != 0 {
            return Ok(None);
        }
        Ok(Some((basis, first)))
    }

    // Pole index (0-based) and basis value of term jj of derivative row ii.
    fn basis_term(
        &self,
        basis: &(Matrix<'static>, i32),
        ii: usize,
        jj: usize,
    ) -> Result<(i64, f64), Failure> {
        let mut index1: i32 = basis.1;
        for _ in 0..jj {
            index1 = index1 % self.modulus + 1;
        }
        let value: f64 = *basis
            .0
            .at(ii, jj)
            .map_err(|error| Failure::out_of_range(&error.to_string()))?;
        Ok((i64::from(index1 - 1), value))
    }

    // Row ii of sum_jj Poles(index1) [* Weights(index1)] * Basis(ii, jj).
    fn combine(
        &self,
        basis: &(Matrix<'static>, i32),
        ii: usize,
        poles: &[f64],
        weights: Option<&[f64]>,
    ) -> Result<Vec<f64>, Failure> {
        let dim: usize = self.dimension;
        let mut row: Vec<f64> = vec![0.0e0; dim];
        for jj in 0..self.order {
            let (pole, term) = self.basis_term(basis, ii, jj)?;
            for (kk, value) in row.iter_mut().enumerate() {
                let coordinate: f64 = buffer(poles, pole * dim as i64 + kk as i64)?;
                *value += match weights {
                    None => coordinate * term,
                    Some(weights) => coordinate * buffer(weights, pole)? * term,
                };
            }
        }
        Ok(row)
    }

    // Taylor coefficients of the poles (row ii divided by ii!), in a buffer large
    // enough for the degree-sized read that follows; the unread native tail is zero.
    fn taylor_poles(
        &self,
        basis: &(Matrix<'static>, i32),
        poles: &[f64],
        weights: Option<&[f64]>,
    ) -> Result<Vec<f64>, Failure> {
        let dim: usize = self.dimension;
        let rows: usize = self.local_request + 1;
        let mut taylor: Vec<f64> = vec![0.0; rows.max(self.order) * dim.max(1)];
        let mut inverse: f64 = 1.0e0;
        for ii in 0..rows {
            let row: Vec<f64> = self.combine(basis, ii, poles, weights)?;
            for (kk, value) in row.into_iter().enumerate() {
                taylor[ii * dim + kk] = value * inverse;
            }
            inverse /= (ii + 1) as f64;
        }
        Ok(taylor)
    }

    // The weight Taylor coefficients overwrite the first LocalRequest+1 values.
    fn taylor_weights(
        &self,
        basis: &(Matrix<'static>, i32),
        weights: &[f64],
        taylor: &mut [f64],
    ) -> Result<(), Failure> {
        let mut inverse: f64 = 1.0e0;
        for (ii, coefficient) in taylor.iter_mut().enumerate().take(self.local_request + 1) {
            *coefficient = 0.0e0;
            for jj in 0..self.order {
                let (pole, term) = self.basis_term(basis, ii, jj)?;
                *coefficient += buffer(weights, pole)? * term;
            }
            *coefficient *= inverse;
            inverse /= (ii + 1) as f64;
        }
        Ok(())
    }

    fn evaluate_taylor(
        &self,
        taylor: &[f64],
        dim: usize,
        results: &mut [f64],
    ) -> Result<(), Failure> {
        let new_request: i32 = self.derivative_request.min(self.degree);
        eval_polynomial(
            self.delta,
            new_request,
            self.degree,
            dim as i32,
            taylor,
            results,
        )
    }
}

fn buffer(values: &[f64], index: i64) -> Result<f64, Failure> {
    crate::plib::access::buffer_value(EVAL, values, index)
}

// Brings a periodic parameter into [FlatKnots(2), FlatKnots(Upper-1)] by whole periods.
fn periodic_parameter(parameter: f64, first: f64, last: f64) -> Result<f64, Failure> {
    let period: f64 = last - first;
    let mut local: f64 = parameter;
    let needs_shift: bool = local > last || local < first;
    // A non-positive period would make the native loops spin forever.
    if needs_shift && !(period > 0.0 && period.is_finite() && local.is_finite()) {
        return Err(Failure::out_of_range(&format!(
            "{EVAL}: period {period} cannot bring {parameter} into [{first}, {last}]"
        )));
    }
    while local > last {
        local -= period;
    }
    while local < first {
        local += period;
    }
    Ok(local)
}

// BSplCLib_LocalMatrix: a 27x27 stack buffer.
fn local_matrix_check(local_request: i32, order: i32) -> Result<(), Failure> {
    if local_request > MAX_DEGREE || order > MAX_DEGREE + 1 {
        return Err(Failure::out_of_range(
            "BSplCLib: bspline degree is greater than maximum supported",
        ));
    }
    Ok(())
}

/// Evaluates a 2D or 3D point and its weight from flat knots (`BSplCLib::Eval` on
/// gp_Pnt/gp_Pnt2d); with `homogeneous_flag` the point is the weighted numerator,
/// otherwise it is divided by the weight;
/// `bsplclib::eval_extrapolated_point(0.5, false, false, &[3, 3], 3, &flat, &poles, &w)?`.
#[allow(clippy::too_many_arguments)] // Mirrors the native ten-argument signature.
pub fn eval_extrapolated_point<P: PolePoint>(
    parameter: f64,
    periodic_flag: bool,
    homogeneous_flag: bool,
    extrap_mode: &[i32; 2],
    degree: i32,
    flat_knots: &Array1<'_, f64>,
    poles: &Array1<'_, P>,
    weights: &Array1<'_, f64>,
    point: &mut P,
    weight: &mut f64,
) -> Result<(), Failure> {
    let flat: Vec<f64> = flat_points(poles);
    let dim: i32 = P::DIMENSION as i32;
    let mut p: Vec<f64> = vec![0.0; P::DIMENSION];
    let mut w: [f64; 1] = [*weight];
    if homogeneous_flag {
        eval_extrapolated(
            parameter,
            periodic_flag,
            0,
            extrap_mode,
            degree,
            flat_knots,
            dim,
            &flat,
            &mut p,
        )?;
        eval_extrapolated(
            parameter,
            periodic_flag,
            0,
            extrap_mode,
            degree,
            flat_knots,
            1,
            weights.as_slice(),
            &mut w,
        )?;
    } else {
        eval_extrapolated_rational(
            parameter,
            periodic_flag,
            0,
            extrap_mode,
            degree,
            flat_knots,
            dim,
            &flat,
            weights.as_slice(),
            &mut p,
            &mut w,
        )?;
        let inverse: f64 = 1.0e0 / w[0];
        for value in &mut p {
            *value *= inverse;
        }
    }
    *weight = w[0];
    for (k, value) in p.into_iter().enumerate() {
        point.set_pole_coordinate(k, value);
    }
    Ok(())
}
