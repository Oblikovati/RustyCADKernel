//! Direct math_FRPR_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut, unused_variables)]
use rustycad_tkmath::math::{Frpr, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct QuadraticBowlFunction;
impl MultipleVarFunction for QuadraticBowlFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let dx: f64 = point[1] - 1.0;
        let dy: f64 = point[2] - 2.0;
        *value = dx * dx + dy * dy;
        true
    }
}
impl MultipleVarFunctionWithGradient for QuadraticBowlFunction {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        gradient[1] = 2.0 * (point[1] - 1.0);
        gradient[2] = 2.0 * (point[2] - 2.0);
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        true
    }
}
struct RosenbrockFunction;
impl MultipleVarFunction for RosenbrockFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let x: f64 = point[1];
        let y: f64 = point[2];
        let dx: f64 = 1.0 - x;
        let dy: f64 = y - x * x;
        *value = dx * dx + 100.0 * dy * dy;
        true
    }
}
impl MultipleVarFunctionWithGradient for RosenbrockFunction {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let x: f64 = point[1];
        let y: f64 = point[2];
        gradient[1] = -2.0 * (1.0 - x) + 200.0 * (y - x * x) * (-2.0 * x);
        gradient[2] = 200.0 * (y - x * x);
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        true
    }
}
struct Quadratic3DFunction;
impl MultipleVarFunction for Quadratic3DFunction {
    fn variables(&self) -> i32 {
        3
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let dx: f64 = point[1] - 1.0;
        let dy: f64 = point[2] - 2.0;
        let dz: f64 = point[3] - 3.0;
        *value = dx * dx + 2.0 * dy * dy + 3.0 * dz * dz;
        true
    }
}
impl MultipleVarFunctionWithGradient for Quadratic3DFunction {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        gradient[1] = 2.0 * (point[1] - 1.0);
        gradient[2] = 4.0 * (point[2] - 2.0);
        gradient[3] = 6.0 * (point[3] - 3.0);
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        true
    }
}
struct LinearFunction;
impl MultipleVarFunction for LinearFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        *value = 2.0 * point[1] + 3.0 * point[2];
        true
    }
}
impl MultipleVarFunctionWithGradient for LinearFunction {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let _ = point;
        gradient[1] = 2.0;
        gradient[2] = 3.0;
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        true
    }
}
struct QuarticFunction;
impl MultipleVarFunction for QuarticFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        let dx: f64 = point[1] - 1.0;
        let dy: f64 = point[2] - 2.0;
        *value = dx * dx * dx * dx + dy * dy * dy * dy;
        true
    }
}
impl MultipleVarFunctionWithGradient for QuarticFunction {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        let dx: f64 = point[1] - 1.0;
        let dy: f64 = point[2] - 2.0;
        gradient[1] = 4.0 * dx * dx * dx;
        gradient[2] = 4.0 * dy * dy * dy;
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value);
        self.gradient(point, gradient);
        true
    }
}
include!("math_frpr/suites_0.rs");
include!("math_frpr/suites_1.rs");
include!("math_frpr/suites_2.rs");
include!("math_frpr/suites_3.rs");
include!("math_frpr/suites_4.rs");
