// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_EvaluatorFunc2Var.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_EvaluatorFunc2Var`: the function of two parameters an approximation
//! samples.
//!
//! The native virtual `Evaluate` passes every argument through a pointer; no kernel
//! reads an input back after the call, so the inputs are values here. The error
//! code stays an in-out argument: the kernels clear it before the call and test it
//! after, and an evaluator that never writes it leaves the kernel's value, as
//! natively. `operator()` is the same call. The trait takes `&self` like the
//! native `const` member.

/// A function `F(U, V)` with values in a space of `dimension` reals.
///
/// Closures with the matching signature implement it;
/// `let f = |_: &AdvApp2VarEvaluation<'_>, r: &mut [f64], e: &mut i32| { r.fill(0.0); *e = 0; };`.
pub trait AdvApp2VarEvaluatorFunc2Var {
    /// Evaluates the derivative `(u_order, v_order)` of `F` on the iso of constant
    /// parameter `const_param` (U when `favor_iso` is 1, V otherwise) at the
    /// `nb_params` values `parameters` of the other parameter, into
    /// `result[dimension * nb_params]`; a positive `error_code` stops the kernel
    /// (`Evaluate`). `f.evaluate(&request, &mut values, &mut code)`.
    fn evaluate(
        &self,
        request: &AdvApp2VarEvaluation<'_>,
        result: &mut [f64],
        error_code: &mut i32,
    );
}

/// The inputs of one `Evaluate` call, in the native argument order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdvApp2VarEvaluation<'a> {
    /// `theDimension`: number of reals per value.
    pub dimension: i32,
    /// `theUStartEnd`: first and last parameters in U.
    pub u_start_end: [f64; 2],
    /// `theVStartEnd`: first and last parameters in V.
    pub v_start_end: [f64; 2],
    /// `theFavorIso`: 1 when U is constant, 2 when V is.
    pub favor_iso: i32,
    /// `theConstParam`: the value of the constant parameter.
    pub const_param: f64,
    /// `theNbParams`: the number of parameter values.
    pub nb_params: i32,
    /// `theParameters`: the values of the other parameter.
    pub parameters: &'a [f64],
    /// `theUOrder`: derivative order in U.
    pub u_order: i32,
    /// `theVOrder`: derivative order in V.
    pub v_order: i32,
}

impl<F> AdvApp2VarEvaluatorFunc2Var for F
where
    F: Fn(&AdvApp2VarEvaluation<'_>, &mut [f64], &mut i32),
{
    fn evaluate(
        &self,
        request: &AdvApp2VarEvaluation<'_>,
        result: &mut [f64],
        error_code: &mut i32,
    ) {
        self(request, result, error_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closures_are_evaluators_and_may_leave_the_error_code() {
        let f = |request: &AdvApp2VarEvaluation<'_>, result: &mut [f64], _: &mut i32| {
            for (value, t) in result.iter_mut().zip(request.parameters) {
                *value = request.const_param + t;
            }
        };
        let request = AdvApp2VarEvaluation {
            dimension: 1,
            u_start_end: [0.0, 1.0],
            v_start_end: [0.0, 1.0],
            favor_iso: 1,
            const_param: 0.5,
            nb_params: 2,
            parameters: &[0.25, 0.75],
            u_order: 0,
            v_order: 0,
        };
        let (mut result, mut code) = ([0.0; 2], -4);
        f.evaluate(&request, &mut result, &mut code);
        assert_eq!((result, code), ([0.75, 1.25], -4));
    }
}
