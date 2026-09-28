// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/AdvApprox/AdvApprox_ApproxAFunction.hxx/.cxx/.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApprox_ApproxAFunction`: approximation of a function with values in a product
//! of 1D, 2D and 3D subspaces by one B-spline per subspace, sharing knots.
use super::approximation::{
    AdvApproxApproximationInput, AdvApproxApproximationOutput, approximation,
};
use super::cutting::{AdvApproxCutting, AdvApproxDichoCutting};
use super::evaluator::AdvApproxEvaluatorFunction;
use super::prepare_convert::{Pieces, prepare_convert};
use super::results::ConvertedResult;
use crate::geom::errors;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays::{at, put};
use rustycad_tkmath::convert::{CompPolynomialToPoles, PolynomialSpans};
use rustycad_tkmath::geom_abs::Shape;

/// A finished (or failed) approximation: shared knots, degree and the poles and
/// errors of each subspace; the native constructors run the whole computation.
///
/// `AdvApproxApproxAFunction::new(0, 0, 1, None, None, Some(&tol3d), 0.0, 1.0,
/// Shape::C1, 14, 100, &mut f)?.poles()`.
pub struct AdvApproxApproxAFunction {
    pub(super) num_sub_spaces: [i32; 3],
    pub(super) first: f64,
    pub(super) last: f64,
    pub(super) continuity: Shape,
    pub(super) max_degree: i32,
    pub(super) max_segments: i32,
    pub(super) done: bool,
    pub(super) has_result: bool,
    pub(super) result: ConvertedResult,
}

/// Tolerance tables of the 1D, 2D and 3D subspaces (the native handles, possibly null).
pub type SubSpaceTolerances<'r, 'a> = (
    Option<&'r Array1<'a, f64>>,
    Option<&'r Array1<'a, f64>>,
    Option<&'r Array1<'a, f64>>,
);

// The Approximation result of Perform, before the B-spline conversion.
struct Pieced {
    num_curves: i32,
    num_coeff: Array1<'static, i32>,
    coefficients: Array1<'static, f64>,
    intervals: Array1<'static, f64>,
    error_max: Array1<'static, f64>,
    average_error: Array1<'static, f64>,
    error_code: i32,
}

fn one_based<T: Default + Clone>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count).map_err(errors::array_bounds)
}

impl AdvApproxApproxAFunction {
    /// Approximates `func` on `[first, last]` with dichotomic cutting; tolerances are
    /// read at `1..=NumXDSS`. Standard_ConstructionError for inconsistent counts,
    /// `last < first`, `max_deg < 1`, `max_seg < 0` or a continuity beyond C2
    /// (`AdvApprox_ApproxAFunction(Num1DSS, Num2DSS, Num3DSS, OneDTol, TwoDTol,
    /// ThreeDTol, First, Last, Continuity, MaxDeg, MaxSeg, Func)`);
    /// `AdvApproxApproxAFunction::new(1, 0, 0, Some(&tol), None, None, 0.0, 1.0, Shape::C1, 14, 100, &mut f)?`.
    #[allow(clippy::too_many_arguments)] // The native member list.
    pub fn new(
        num_1d_ss: i32,
        num_2d_ss: i32,
        num_3d_ss: i32,
        one_d_tol: Option<&Array1<'_, f64>>,
        two_d_tol: Option<&Array1<'_, f64>>,
        three_d_tol: Option<&Array1<'_, f64>>,
        first: f64,
        last: f64,
        continuity: Shape,
        max_deg: i32,
        max_seg: i32,
        func: &mut dyn AdvApproxEvaluatorFunction,
    ) -> Result<Self, Failure> {
        let spaces = [num_1d_ss, num_2d_ss, num_3d_ss];
        let tolerances = (one_d_tol, two_d_tol, three_d_tol);
        let mut approx = Self::unperformed(first, last, continuity, max_deg, max_seg);
        approx.perform(spaces, tolerances, func, &AdvApproxDichoCutting)?;
        Ok(approx)
    }

    /// The same with a cutting strategy (`AdvApprox_ApproxAFunction(..., Func, CutTool)`);
    /// `AdvApproxApproxAFunction::with_cutting(0, 0, 1, None, None, Some(&t), 0.0, 1.0, Shape::C1, 14, 100, &mut f, &cut)?`.
    #[allow(clippy::too_many_arguments)] // The native member list.
    pub fn with_cutting(
        num_1d_ss: i32,
        num_2d_ss: i32,
        num_3d_ss: i32,
        one_d_tol: Option<&Array1<'_, f64>>,
        two_d_tol: Option<&Array1<'_, f64>>,
        three_d_tol: Option<&Array1<'_, f64>>,
        first: f64,
        last: f64,
        continuity: Shape,
        max_deg: i32,
        max_seg: i32,
        func: &mut dyn AdvApproxEvaluatorFunction,
        cut_tool: &dyn AdvApproxCutting,
    ) -> Result<Self, Failure> {
        let spaces = [num_1d_ss, num_2d_ss, num_3d_ss];
        let tolerances = (one_d_tol, two_d_tol, three_d_tol);
        let mut approx = Self::unperformed(first, last, continuity, max_deg, max_seg);
        approx.perform(spaces, tolerances, func, cut_tool)?;
        Ok(approx)
    }

    fn unperformed(first: f64, last: f64, continuity: Shape, max_deg: i32, max_seg: i32) -> Self {
        Self {
            num_sub_spaces: [0; 3],
            first,
            last,
            continuity,
            max_degree: max_deg,
            max_segments: max_seg,
            done: false,
            has_result: false,
            result: ConvertedResult::default(),
        }
    }

    /// `Approximation`, the static piecewise-polynomial engine the constructors use;
    /// see [`approximation`](super::approximation::approximation).
    pub fn approximation(
        input: &AdvApproxApproximationInput<'_, '_>,
        evaluator: &mut dyn AdvApproxEvaluatorFunction,
        cut_tool: &dyn AdvApproxCutting,
        output: &mut AdvApproxApproximationOutput<'_, '_>,
    ) -> Result<(), Failure> {
        approximation(input, evaluator, cut_tool, output)
    }

    // The private native Perform: checks, Approximation, continuity analysis and
    // conversion of the pieces into one B-spline per subspace.
    fn perform(
        &mut self,
        spaces: [i32; 3],
        tolerances: SubSpaceTolerances<'_, '_>,
        func: &mut dyn AdvApproxEvaluatorFunction,
        cut_tool: &dyn AdvApproxCutting,
    ) -> Result<(), Failure> {
        self.check_input(spaces)?;
        self.max_degree = self.max_degree.min(14);
        self.num_sub_spaces = spaces;
        let continuity_order: i32 = self.continuity_order()?;
        let num_max_coeffs: i32 = (self.max_degree + 1).max(2 * continuity_order + 2);
        self.max_degree = num_max_coeffs - 1;
        let (local_dimension, local_tolerances) = local_tables(spaces, tolerances)?;
        let pieced = self.pieces(
            &local_dimension,
            &local_tolerances,
            continuity_order,
            func,
            cut_tool,
        )?;
        if pieced.error_code != 0 && pieced.error_code != -1 {
            return Ok(());
        }
        self.convert(pieced, continuity_order, &local_tolerances)
    }

    fn check_input(&self, [n1, n2, n3]: [i32; 3]) -> Result<(), Failure> {
        if n1 < 0
            || n2 < 0
            || n3 < 0
            || n1 + n2 + n3 <= 0
            || self.last < self.first
            || self.max_degree < 1
            || self.max_segments < 0
        {
            return Err(errors::construction(&format!(
                "AdvApprox_ApproxAFunction: Num1DSS = {n1}, Num2DSS = {n2}, Num3DSS = {n3}, First = {}, Last = {}, MaxDeg = {}, MaxSeg = {}; expected non-negative counts with a positive sum, First <= Last, MaxDeg >= 1 and MaxSeg >= 0",
                self.first, self.last, self.max_degree, self.max_segments
            )));
        }
        Ok(())
    }

    fn continuity_order(&self) -> Result<i32, Failure> {
        match self.continuity {
            Shape::C0 => Ok(0),
            Shape::C1 => Ok(1),
            Shape::C2 => Ok(2),
            other => Err(errors::construction(&format!(
                "AdvApprox_ApproxAFunction: Continuity = {other:?}; expected C0, C1 or C2"
            ))),
        }
    }

    fn pieces(
        &self,
        local_dimension: &Array1<'static, i32>,
        local_tolerances: &Array1<'static, f64>,
        continuity_order: i32,
        func: &mut dyn AdvApproxEvaluatorFunction,
        cut_tool: &dyn AdvApproxCutting,
    ) -> Result<Pieced, Failure> {
        let [n1, n2, n3] = self.num_sub_spaces;
        let (total_num_ss, total_dimension) = (n1 + n2 + n3, n1 + 2 * n2 + 3 * n3);
        let num_max_coeffs: i32 = self.max_degree + 1;
        let segments: i32 = self.max_segments;
        let mut pieced = Pieced {
            num_curves: 0,
            num_coeff: one_based(segments)?,
            coefficients: one_based(segments * num_max_coeffs * total_dimension)?,
            intervals: one_based(segments + 1)?,
            error_max: one_based(segments * total_num_ss)?,
            average_error: one_based(segments * total_num_ss)?,
            error_code: 0,
        };
        let input = AdvApproxApproximationInput {
            total_dimension,
            total_num_ss,
            local_dimension,
            first: self.first,
            last: self.last,
            continuity_order,
            num_max_coeffs,
            max_segments: segments,
            tolerances: local_tolerances,
            code_precis: 1,
        };
        let mut output = AdvApproxApproximationOutput {
            num_curves: 0,
            num_coeff_per_curve: &mut pieced.num_coeff,
            coefficients: &mut pieced.coefficients,
            intervals: &mut pieced.intervals,
            error_max: &mut pieced.error_max,
            average_error: &mut pieced.average_error,
            error_code: 0,
        };
        approximation(&input, func, cut_tool, &mut output)?;
        let (num_curves, error_code) = (output.num_curves, output.error_code);
        pieced.num_curves = num_curves;
        pieced.error_code = error_code;
        Ok(pieced)
    }

    // The pieces live on [-1, 1] ("the Taylor expansion at 0"); each has at least
    // two coefficients (PRO5474) before the continuity analysis and the conversion.
    fn convert(
        &mut self,
        mut pieced: Pieced,
        continuity_order: i32,
        local_tolerances: &Array1<'static, f64>,
    ) -> Result<(), Failure> {
        let num_curves: i32 = pieced.num_curves;
        let mut polynomial_intervals: Array2<'static, f64> =
            Array2::with_bounds(1, num_curves, 1, 2).map_err(errors::array_bounds)?;
        for ii in 1..=num_curves {
            let count: i32 = at(&pieced.num_coeff, ii)?.max(2);
            put(&mut pieced.num_coeff, ii, count)?;
            polynomial_intervals
                .set_value(ii, 1, -1.0)
                .map_err(errors::array_bounds)?;
            polynomial_intervals
                .set_value(ii, 2, 1.0)
                .map_err(errors::array_bounds)?;
        }
        let mut tab_continuity: Array1<'static, i32> = one_based(num_curves)?;
        let pieces = Pieces {
            num_coeff_per_curve: &pieced.num_coeff,
            coefficients: &pieced.coefficients,
            polynomial_intervals: &polynomial_intervals,
            true_intervals: &pieced.intervals,
        };
        let [n1, n2, n3] = self.num_sub_spaces;
        prepare_convert(
            num_curves,
            self.max_degree,
            continuity_order,
            (n1, n2, n3),
            &pieces,
            local_tolerances,
            &mut pieced.error_max,
            &mut tab_continuity,
        )?;
        let spans = PolynomialSpans {
            num_coeff_per_curve: &pieced.num_coeff,
            coefficients: &pieced.coefficients,
            polynomial_intervals: &polynomial_intervals,
            true_intervals: &pieced.intervals,
        };
        let total_dimension: i32 = n1 + 2 * n2 + 3 * n3;
        let converter = CompPolynomialToPoles::from_continuities(
            num_curves,
            total_dimension,
            self.max_degree,
            &tab_continuity,
            spans,
        )?;
        if !converter.is_done() {
            return Ok(());
        }
        let errors = (&pieced.error_max, &pieced.average_error);
        self.result =
            ConvertedResult::extract(&converter, self.num_sub_spaces, num_curves, errors)?;
        self.done = pieced.error_code == 0;
        self.has_result = true;
        Ok(())
    }
}

// Dimension and tolerance of each subspace, 1D first, then 2D, then 3D.
fn local_tables(
    spaces: [i32; 3],
    (one, two, three): SubSpaceTolerances<'_, '_>,
) -> Result<(Array1<'static, i32>, Array1<'static, f64>), Failure> {
    let total_num_ss: i32 = spaces.iter().sum();
    let mut dimensions: Array1<'static, i32> = one_based(total_num_ss)?;
    let mut tolerances: Array1<'static, f64> = one_based(total_num_ss)?;
    let mut index: i32 = 1;
    for (dimension, table) in (1..).zip([one, two, three]) {
        for jj in 1..=spaces[dimension as usize - 1] {
            let Some(table) = table else {
                return Err(Failure::with_type(
                    "Standard_NullObject",
                    &format!(
                        "AdvApprox_ApproxAFunction: null {dimension}D tolerance table; expected {} values",
                        spaces[dimension as usize - 1]
                    ),
                    "",
                ));
            };
            put(&mut tolerances, index, at(table, jj)?)?;
            put(&mut dimensions, index, dimension)?;
            index += 1;
        }
    }
    Ok((dimensions, tolerances))
}
