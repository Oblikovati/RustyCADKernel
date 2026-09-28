// Copyright (c) 2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BSplSLib/BSplSLib_Cache.cxx (EvaluatePolynomials) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Evaluation of the cached bivariate polynomial and its derivatives.
use super::{Cache, eval_polynomial, no_derivative_eval_polynomial, rational_derivative};
use rustycad_tkernel::standard::Failure;

impl Cache {
    /// Derivatives up to (`[u, v]` orders) at normalized parameters, row by row along
    /// the max-degree direction, `(min order + 1)` points per row (rational ones
    /// converted to Cartesian derivatives); non-rational rows skip derivatives the
    /// D1/D2 layouts never read; `cache.evaluate_polynomials(0.0, 0.0, [1, 1], &mut out)?`.
    pub(super) fn evaluate_polynomials(
        &self,
        local_u: f64,
        local_v: f64,
        [u_deriv_max, v_deriv_max]: [i32; 2],
        result: &mut [f64; 36],
    ) -> Result<(), Failure> {
        let dimension: usize = if self.is_rational { 4 } else { 3 };
        let cols: usize = self.poles_weights.nb_columns();
        let is_max_u: bool = self.u_is_max();
        let (min_param, max_param) = if is_max_u {
            (local_v, local_u)
        } else {
            (local_u, local_v)
        };
        let (max_deriv, min_deriv) = if is_max_u {
            (u_deriv_max, v_deriv_max)
        } else {
            (v_deriv_max, u_deriv_max)
        };
        let (min_degree, max_degree) = self.degrees();
        let row_stride: usize = (min_deriv as usize + 1) * dimension;
        let mut transient: Vec<f64> =
            vec![0.0; ((max_deriv as usize + 1) * cols).max((min_deriv as usize + 1) * dimension)];
        // Along the variable of maximal degree.
        eval_polynomial(
            max_param,
            max_deriv,
            max_degree,
            cols as i32,
            self.poles_weights.as_slice(),
            &mut transient,
        )?;
        let dim_i32: i32 = dimension as i32;
        // Row 0: derivatives along the variable of minimal degree.
        eval_polynomial(
            min_param,
            min_deriv,
            min_degree,
            dim_i32,
            &transient,
            &mut result[..],
        )?;
        if max_deriv > 0 {
            // Row 1: non-rational layouts only need (1,0) and (1,1).
            let order: i32 = if self.is_rational {
                min_deriv
            } else {
                min_deriv.min(1)
            };
            eval_polynomial(
                min_param,
                order,
                min_degree,
                dim_i32,
                &transient[cols..],
                &mut result[row_stride..],
            )?;
            if max_deriv > 1 {
                self.third_row(
                    min_param,
                    min_deriv,
                    min_degree,
                    &transient[2 * cols..],
                    &mut result[2 * row_stride..],
                )?;
            }
        }
        if !self.is_rational {
            return Ok(());
        }
        // Rows are laid out along the max-degree direction; RationalDerivative is told so.
        let size: usize = ((u_deriv_max + 1) * (v_deriv_max + 1) * 3) as usize;
        let mut rational: Vec<f64> = vec![0.0; size];
        let (n, m) = if is_max_u {
            (u_deriv_max, v_deriv_max)
        } else {
            (v_deriv_max, u_deriv_max)
        };
        rational_derivative(n, m, n, m, &result[..], &mut rational, true)?;
        result[..size].copy_from_slice(&rational);
        Ok(())
    }

    // Row 2: the value only for non-rational layouts, the full row otherwise.
    fn third_row(
        &self,
        min_param: f64,
        min_deriv: i32,
        min_degree: i32,
        coefficients: &[f64],
        row: &mut [f64],
    ) -> Result<(), Failure> {
        let dimension: i32 = if self.is_rational { 4 } else { 3 };
        let order: i32 = if self.is_rational { min_deriv } else { 0 };
        if order == 0 {
            return no_derivative_eval_polynomial(
                min_param,
                min_degree,
                dimension,
                min_degree * dimension,
                coefficients,
                row,
            );
        }
        eval_polynomial(min_param, order, min_degree, dimension, coefficients, row)
    }
}
