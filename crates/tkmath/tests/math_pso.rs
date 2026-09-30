//! Direct math_PSO_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::math::{MultipleVarFunction, Pso, PsoParticlesPool, Vector};
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
struct Quadratic1DFunction;
impl MultipleVarFunction for Quadratic1DFunction {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        let dx: f64 = x[1] - 3.0;
        *value = dx * dx;
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
struct MultiModalFunction;
impl MultipleVarFunction for MultiModalFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        let dx: f64 = x[1] - PI;
        let dy: f64 = x[2] - PI;
        *value = -x[1].cos() * x[2].cos() * (-(dx * dx + dy * dy)).exp();
        true
    }
}
struct Quadratic3DFunction;
impl MultipleVarFunction for Quadratic3DFunction {
    fn variables(&self) -> i32 {
        3
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        *value = x[1] * x[1] + 2.0 * x[2] * x[2] + 3.0 * x[3] * x[3];
        true
    }
}
include!("math_pso/suites_0.rs");
include!("math_pso/suites_1.rs");
include!("math_pso/suites_2.rs");
include!("math_pso/suites_3.rs");
