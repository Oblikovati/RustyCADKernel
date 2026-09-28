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
// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib_EvaluatorFunction.hxx, BSplCLib_2.cxx (FunctionMultiply, FunctionReparameterise) and BSplCLib_CurveComputation.pxx (point overloads) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Products `a(t) F(t)` and compositions `F(a(t))` of a B-spline with a function,
//! re-interpolated at the Schoenberg points of the new flat knots.
//!
//! The native code passes a single `int` where Eval reads the two extrapolation
//! modes, so the second mode comes from unrelated stack memory; the port uses the
//! B-spline degree for both, which disables extrapolation at either end.
use super::extrapolate::eval_extrapolated;
use super::interpolate::{build_schoenberg_points, interpolate_flat};
use super::point::CurvePoint;
use super::views::one_based;
use super::{flat_points, store_flat_points};
use crate::plib::access::{array_cell, array_value, require_length};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

const MULTIPLY: &str = "BSplCLib::FunctionMultiply";

/// A scalar function a(t) and its derivatives (`BSplCLib_EvaluatorFunction`).
pub trait EvaluatorFunction {
    /// Evaluates derivative `derivative_request` at `parameter` within
    /// `start_end`, setting `error_code` non-zero on failure;
    /// `function.evaluate(0, &[0.0, 1.0], 0.5, &mut value, &mut code)`.
    fn evaluate(
        &self,
        derivative_request: i32,
        start_end: &[f64; 2],
        parameter: f64,
        result: &mut f64,
        error_code: &mut i32,
    );
}

// Shared setup: the new pole count, the [start, end] range and Schoenberg parameters.
struct Resampling {
    num_new_poles: i32,
    start_end: [f64; 2],
    parameters: Array1<'static, f64>,
    contact_orders: Array1<'static, i32>,
}

impl Resampling {
    fn new(flat_knots: &Array1<'_, f64>, new_degree: i32) -> Result<Self, Failure> {
        let num_new_poles: i32 = flat_knots.len() as i32 - new_degree - 1;
        let start_end: [f64; 2] = [
            array_value(flat_knots, new_degree + 1)?,
            array_value(flat_knots, num_new_poles + 1)?,
        ];
        let mut parameters: Array1<'static, f64> = one_based(MULTIPLY, num_new_poles)?;
        build_schoenberg_points(new_degree, flat_knots, &mut parameters)?;
        Ok(Self {
            num_new_poles,
            start_end,
            parameters,
            contact_orders: one_based(MULTIPLY, num_new_poles)?,
        })
    }

    // Clamps the first and last parameter into [start, end] (FunctionMultiply only).
    fn clamp_ends(&mut self) -> Result<(), Failure> {
        let n: i32 = self.num_new_poles;
        if array_value(&self.parameters, 1)? < self.start_end[0] {
            *array_cell(&mut self.parameters, 1)? = self.start_end[0];
        }
        if array_value(&self.parameters, n)? > self.start_end[1] {
            *array_cell(&mut self.parameters, n)? = self.start_end[1];
        }
        Ok(())
    }

    // a(t_i), or None after setting the status on an evaluation error.
    fn function_value(
        &mut self,
        function: &dyn EvaluatorFunction,
        i: i32,
    ) -> Result<Option<f64>, Failure> {
        *array_cell(&mut self.contact_orders, i)? = 0;
        let (mut result, mut error_code): (f64, i32) = (0.0, 0);
        let parameter: f64 = array_value(&self.parameters, i)?;
        function.evaluate(0, &self.start_end, parameter, &mut result, &mut error_code);
        if error_code != 0 {
            return Ok(None);
        }
        Ok(Some(result))
    }
}

/// The B-spline to transform and the new basis.
pub struct FunctionTarget<'r, 'a> {
    /// Degree of F.
    pub bspline_degree: i32,
    /// Flat knots of F.
    pub bspline_flat_knots: &'r Array1<'a, f64>,
    /// Flat knots of the result.
    pub flat_knots: &'r Array1<'a, f64>,
    /// Degree of the result.
    pub new_degree: i32,
}

/// Poles of `a(t) F(t)` with F given by `poles_dimension`-value rows
/// (`BSplCLib::FunctionMultiply` on reals); `status` is 1 when `a` fails, else the
/// interpolation pivot problem (0 when fine);
/// `bsplclib::function_multiply_flat(&f, &target, 3, &poles, &mut new_poles, &mut status)?`.
pub fn function_multiply_flat(
    function: &dyn EvaluatorFunction,
    target: &FunctionTarget<'_, '_>,
    poles_dimension: i32,
    poles: &[f64],
    new_poles: &mut [f64],
    status: &mut i32,
) -> Result<(), Failure> {
    let mut sampling = Resampling::new(target.flat_knots, target.new_degree)?;
    sampling.clamp_ends()?;
    let dim: usize = poles_dimension.max(0) as usize;
    let mut values: Vec<f64> = vec![0.0; sampling.num_new_poles.max(0) as usize * dim];
    let mode: [i32; 2] = [target.bspline_degree, target.bspline_degree];
    for i in 1..=sampling.num_new_poles {
        let Some(factor) = sampling.function_value(function, i)? else {
            *status = 1;
            return Ok(());
        };
        let parameter: f64 = array_value(&sampling.parameters, i)?;
        let row: &mut [f64] = &mut values[(i - 1) as usize * dim..i as usize * dim];
        eval_extrapolated(
            parameter,
            false,
            0,
            &mode,
            target.bspline_degree,
            target.bspline_flat_knots,
            poles_dimension,
            poles,
            row,
        )?;
        for value in row.iter_mut() {
            *value *= factor;
        }
    }
    finish(
        target,
        &sampling,
        poles_dimension,
        values,
        new_poles,
        status,
    )
}

// Interpolates the sampled values and copies the new poles.
fn finish(
    target: &FunctionTarget<'_, '_>,
    sampling: &Resampling,
    poles_dimension: i32,
    mut values: Vec<f64>,
    new_poles: &mut [f64],
    status: &mut i32,
) -> Result<(), Failure> {
    interpolate_flat(
        target.new_degree,
        target.flat_knots,
        &sampling.parameters,
        &sampling.contact_orders,
        poles_dimension,
        &mut values,
        status,
    )?;
    require_length(MULTIPLY, "NewPoles", new_poles.len(), values.len())?;
    new_poles[..values.len()].copy_from_slice(&values);
    Ok(())
}

/// Poles of `F(a(t))` (`BSplCLib::FunctionReparameterise` on reals); `status` as for
/// [`function_multiply_flat`];
/// `bsplclib::function_reparameterise_flat(&a, &target, 3, &poles, &mut new_poles, &mut status)?`.
pub fn function_reparameterise_flat(
    function: &dyn EvaluatorFunction,
    target: &FunctionTarget<'_, '_>,
    poles_dimension: i32,
    poles: &[f64],
    new_poles: &mut [f64],
    status: &mut i32,
) -> Result<(), Failure> {
    let mut sampling = Resampling::new(target.flat_knots, target.new_degree)?;
    let dim: usize = poles_dimension.max(0) as usize;
    let mut values: Vec<f64> = vec![0.0; sampling.num_new_poles.max(0) as usize * dim];
    let mode: [i32; 2] = [target.bspline_degree, target.bspline_degree];
    for i in 1..=sampling.num_new_poles {
        let Some(parameter) = sampling.function_value(function, i)? else {
            *status = 1;
            return Ok(());
        };
        let row: &mut [f64] = &mut values[(i - 1) as usize * dim..i as usize * dim];
        eval_extrapolated(
            parameter,
            false,
            0,
            &mode,
            target.bspline_degree,
            target.bspline_flat_knots,
            poles_dimension,
            poles,
            row,
        )?;
    }
    finish(
        target,
        &sampling,
        poles_dimension,
        values,
        new_poles,
        status,
    )
}

// The native pole-count check of the array overloads.
fn check_counts(
    target: &FunctionTarget<'_, '_>,
    poles: usize,
    new_poles: usize,
) -> Result<(), Failure> {
    let expected: i64 =
        target.bspline_flat_knots.len() as i64 - i64::from(target.bspline_degree) - 1;
    let expected_new: i64 = target.flat_knots.len() as i64 - i64::from(target.new_degree) - 1;
    if poles as i64 != expected || new_poles as i64 != expected_new {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            &format!(
                "{MULTIPLY}: {poles} poles and {new_poles} new poles; expected {expected} and {expected_new}"
            ),
            "",
        ));
    }
    Ok(())
}

/// [`function_multiply_flat`] on a scalar pole array (`BSplCLib::FunctionMultiply` on
/// NCollection_Array1<double>); Standard_ConstructionError on pole-count mismatch;
/// `bsplclib::function_multiply_scalar(&a, &target, &poles, &mut new_poles, &mut status)?`.
pub fn function_multiply_scalar(
    function: &dyn EvaluatorFunction,
    target: &FunctionTarget<'_, '_>,
    poles: &Array1<'_, f64>,
    new_poles: &mut Array1<'_, f64>,
    status: &mut i32,
) -> Result<(), Failure> {
    check_counts(target, poles.len(), new_poles.len())?;
    function_multiply_flat(
        function,
        target,
        1,
        poles.as_slice(),
        new_poles.as_mut_slice(),
        status,
    )
}

/// [`function_reparameterise_flat`] on a scalar pole array
/// (`BSplCLib::FunctionReparameterise` on NCollection_Array1<double>);
/// `bsplclib::function_reparameterise_scalar(&a, &target, &poles, &mut new_poles, &mut status)?`.
pub fn function_reparameterise_scalar(
    function: &dyn EvaluatorFunction,
    target: &FunctionTarget<'_, '_>,
    poles: &Array1<'_, f64>,
    new_poles: &mut Array1<'_, f64>,
    status: &mut i32,
) -> Result<(), Failure> {
    check_counts(target, poles.len(), new_poles.len())?;
    function_reparameterise_flat(
        function,
        target,
        1,
        poles.as_slice(),
        new_poles.as_mut_slice(),
        status,
    )
}

/// [`function_multiply_flat`] on 2D/3D poles (`BSplCLib::FunctionMultiply` on
/// gp_Pnt/gp_Pnt2d); `bsplclib::function_multiply(&a, &target, &poles, &mut new_poles, &mut status)?`.
pub fn function_multiply<P: CurvePoint>(
    function: &dyn EvaluatorFunction,
    target: &FunctionTarget<'_, '_>,
    poles: &Array1<'_, P>,
    new_poles: &mut Array1<'_, P>,
    status: &mut i32,
) -> Result<(), Failure> {
    check_counts(target, poles.len(), new_poles.len())?;
    let mut flat_new: Vec<f64> = flat_points(new_poles);
    let dimension: i32 = P::DIMENSION as i32;
    function_multiply_flat(
        function,
        target,
        dimension,
        &flat_points(poles),
        &mut flat_new,
        status,
    )?;
    store_flat_points(&flat_new, new_poles);
    Ok(())
}

/// [`function_reparameterise_flat`] on 2D/3D poles (`BSplCLib::FunctionReparameterise`
/// on gp_Pnt/gp_Pnt2d);
/// `bsplclib::function_reparameterise(&a, &target, &poles, &mut new_poles, &mut status)?`.
pub fn function_reparameterise<P: CurvePoint>(
    function: &dyn EvaluatorFunction,
    target: &FunctionTarget<'_, '_>,
    poles: &Array1<'_, P>,
    new_poles: &mut Array1<'_, P>,
    status: &mut i32,
) -> Result<(), Failure> {
    check_counts(target, poles.len(), new_poles.len())?;
    let mut flat_new: Vec<f64> = flat_points(new_poles);
    let dimension: i32 = P::DIMENSION as i32;
    function_reparameterise_flat(
        function,
        target,
        dimension,
        &flat_points(poles),
        &mut flat_new,
        status,
    )?;
    store_flat_points(&flat_new, new_poles);
    Ok(())
}
