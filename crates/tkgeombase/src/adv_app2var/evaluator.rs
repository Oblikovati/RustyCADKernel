// Rust translation of ModelingData/TKGeomBase/AdvApp2Var/AdvApp2Var_EvaluatorFunc2Var.hxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AdvApp2Var_EvaluatorFunc2Var`: the function of two parameters an approximation
//! samples.
//!
//! The native virtual `Evaluate` passes every argument through a pointer. Legacy
//! evaluators can use the read-only `evaluate` view; `evaluate_native` exposes the
//! mutable pointer protocol to callbacks that need it. `operator()` forwards to
//! the same native method. The trait takes `&self` like the native `const` member.

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

    /// Evaluates with the native mutable pointer arguments. Existing implementations
    /// keep their read-only behavior through this default adapter.
    /// `f.evaluate_native(&mut call, &mut values, &mut code)`.
    fn evaluate_native(
        &self,
        call: &mut AdvApp2VarEvaluationMut<'_>,
        result: &mut [f64],
        error_code: &mut i32,
    ) {
        self.evaluate(&call.snapshot(), result, error_code);
    }
}

/// Mutable arguments passed to one native `Evaluate` call.
pub struct AdvApp2VarEvaluationMut<'a> {
    /// `theDimension`: number of reals per value.
    pub dimension: &'a mut i32,
    /// `theUStartEnd`: first and last parameters in U.
    pub u_start_end: &'a mut [f64; 2],
    /// `theVStartEnd`: first and last parameters in V.
    pub v_start_end: &'a mut [f64; 2],
    /// `theFavorIso`: 1 for constant U, 2 for constant V.
    pub favor_iso: &'a mut i32,
    /// `theConstParam`: the value of the constant parameter.
    pub const_param: &'a mut f64,
    /// `theNbParams`: number of parameters.
    pub nb_params: &'a mut i32,
    /// `theParameters`: mutable parameter storage beginning at the native pointer.
    pub parameters: &'a mut [f64],
    /// `theUOrder`: derivative order in U.
    pub u_order: &'a mut i32,
    /// `theVOrder`: derivative order in V.
    pub v_order: &'a mut i32,
}

impl AdvApp2VarEvaluationMut<'_> {
    fn snapshot(&self) -> AdvApp2VarEvaluation<'_> {
        AdvApp2VarEvaluation {
            dimension: *self.dimension,
            u_start_end: *self.u_start_end,
            v_start_end: *self.v_start_end,
            favor_iso: *self.favor_iso,
            const_param: *self.const_param,
            nb_params: *self.nb_params,
            parameters: self.parameters,
            u_order: *self.u_order,
            v_order: *self.v_order,
        }
    }
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
