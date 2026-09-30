//! Direct math_GlobOptMin_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut, unused_assignments)]
use rustycad_tkmath::math::{
    GlobalObjective, GlobalOptimum, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient,
    MultipleVarFunctionWithHessian, Vector,
};
use std::f64::consts::PI;
fn near(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() <= tolerance, "{a} != {b}");
}
struct QuadraticFunction;
impl MultipleVarFunction for QuadraticFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        let dx: f64 = x[1] - 1.0;
        let dy: f64 = x[2] - 2.0;
        *value = dx * dx + dy * dy;
        true
    }
}
struct MultiModalFunction;
impl MultipleVarFunction for MultiModalFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        *value = x[1].sin() + x[2].sin() + 0.1 * (x[1] * x[1] + x[2] * x[2]);
        true
    }
}
struct MultiModal1DFunction;
impl MultipleVarFunction for MultiModal1DFunction {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        *value = x[1].sin() + 0.5 * (3.0 * x[1]).sin();
        true
    }
}
struct RosenbrockFunction;
impl MultipleVarFunction for RosenbrockFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        let dx: f64 = 1.0 - x[1];
        let dy: f64 = x[2] - x[1] * x[1];
        *value = dx * dx + 100.0 * dy * dy;
        true
    }
}
struct LinearFunction;
impl MultipleVarFunction for LinearFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        *value = x[1] + x[2];
        true
    }
}
include!("math_glob_opt_min/suites_0.rs");
include!("math_glob_opt_min/suites_1.rs");
include!("math_glob_opt_min/suites_2.rs");
include!("math_glob_opt_min/suites_3.rs");
include!("math_glob_opt_min/branin.rs");
