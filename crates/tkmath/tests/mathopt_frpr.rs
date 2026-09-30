//! MathOpt_FRPR_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    clippy::field_reassign_with_default,
    clippy::default_constructed_unit_structs
)]
use rustycad_tkmath::math::{Frpr, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
use rustycad_tkmath::math_opt::{ConjugateGradientFormula, FrprConfig, bfgs, frpr, frpr_numerical};
use rustycad_tkmath::math_utils::{Config, Status, VectorResult};
const THE_TOLERANCE: f64 = 1e-6;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
include!("mathopt_frpr/fixture_0.rs");
include!("mathopt_frpr/fixture_1.rs");
include!("mathopt_frpr/fixture_2.rs");
include!("mathopt_frpr/fixture_3.rs");
include!("mathopt_frpr/fixture_4.rs");
include!("mathopt_frpr/fixture_5.rs");

struct LineFailure {
    nonfinite: bool,
    calls: usize,
}
impl MultipleVarFunction for LineFailure {
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
impl MultipleVarFunctionWithGradient for LineFailure {
    fn gradient(&mut self, x: &Vector<'_>, g: &mut Vector<'_>) -> bool {
        g[0] = 2.0 * x[0];
        true
    }
    fn values(&mut self, x: &Vector<'_>, f: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, f) && self.gradient(x, g)
    }
}
include!("mathopt_frpr/tests_0.rs");
include!("mathopt_frpr/tests_1.rs");
include!("mathopt_frpr/tests_2.rs");
include!("mathopt_frpr/tests_3.rs");
