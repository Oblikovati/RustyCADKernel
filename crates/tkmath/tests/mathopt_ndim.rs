//! MathOpt_NDim_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_mut,
    clippy::field_reassign_with_default,
    clippy::default_constructed_unit_structs,
    clippy::excessive_precision
)]
use rustycad_tkmath::math::{Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
use rustycad_tkmath::math_opt::*;
use rustycad_tkmath::math_utils::{Config, Status, VectorResult};
include!("mathopt_ndim/fixture_0.rs");
include!("mathopt_ndim/fixture_1.rs");
include!("mathopt_ndim/fixture_2.rs");
include!("mathopt_ndim/fixture_3.rs");
include!("mathopt_ndim/fixture_4.rs");
include!("mathopt_ndim/fixture_5.rs");
include!("mathopt_ndim/fixture_9.rs");
include!("mathopt_ndim/fixture_10.rs");
include!("mathopt_ndim/fixture_11.rs");
include!("mathopt_ndim/fixture_12.rs");
include!("mathopt_ndim/fixture_13.rs");
include!("mathopt_ndim/fixture_14.rs");
include!("mathopt_ndim/fixture_15.rs");
include!("mathopt_ndim/fixture_16.rs");

#[derive(Default)]
struct LineSearchFailureFunction {
    nonfinite: bool,
    calls: usize,
}
impl MultipleVarFunction for LineSearchFailureFunction {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        self.calls += 1;
        if self.calls > 1 {
            if !self.nonfinite {
                return false;
            }
            *f = f64::NAN;
            return true;
        }
        *f = x[0] * x[0];
        true
    }
}
impl MultipleVarFunctionWithGradient for LineSearchFailureFunction {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        g[0] = 2.0 * x[0];
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
#[derive(Default)]
struct GradientCallbackFailureFunction;
impl MultipleVarFunction for GradientCallbackFailureFunction {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut f64) -> bool {
        *f = x[0] * x[0];
        true
    }
}
impl MultipleVarFunctionWithGradient for GradientCallbackFailureFunction {
    fn gradient(&mut self, _: &Vector<'_>, _: &mut Vector<'_>) -> bool {
        false
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
#[derive(Default)]
struct ValueCallbackFailureFunction;
impl MultipleVarFunction for ValueCallbackFailureFunction {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, _: &Vector<'_>, _: &mut f64) -> bool {
        false
    }
}
include!("mathopt_ndim/tests_0.rs");
include!("mathopt_ndim/tests_1.rs");
include!("mathopt_ndim/tests_2.rs");
include!("mathopt_ndim/tests_3.rs");
include!("mathopt_ndim/tests_4.rs");
include!("mathopt_ndim/tests_5.rs");
include!("mathopt_ndim/tests_6.rs");

fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
