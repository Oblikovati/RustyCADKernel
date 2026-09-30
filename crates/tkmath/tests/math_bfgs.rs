//! Direct math_BFGS_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut, unused_variables)]
use rustycad_tkmath::math::{
    Bfgs, Frpr, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector,
};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct QuadraticFunction2D;
impl MultipleVarFunction for QuadraticFunction2D {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        if point.length() != 2 {
            return false;
        }
        let dx: f64 = point[1] - 1.0;
        let dy: f64 = point[2] - 2.0;
        *value = dx * dx + dy * dy;
        true
    }
}
impl MultipleVarFunctionWithGradient for QuadraticFunction2D {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        if point.length() != 2 || gradient.length() != 2 {
            return false;
        }
        gradient[1] = 2.0 * (point[1] - 1.0);
        gradient[2] = 2.0 * (point[2] - 2.0);
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
struct RosenbrockFunction;
impl MultipleVarFunction for RosenbrockFunction {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        if point.length() != 2 {
            return false;
        }
        let x: f64 = point[1];
        let y: f64 = point[2];
        let t1: f64 = y - x * x;
        let t2: f64 = 1.0 - x;
        *value = 100.0 * t1 * t1 + t2 * t2;
        true
    }
}
impl MultipleVarFunctionWithGradient for RosenbrockFunction {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        if point.length() != 2 || gradient.length() != 2 {
            return false;
        }
        let x: f64 = point[1];
        let y: f64 = point[2];
        gradient[1] = -400.0 * x * (y - x * x) - 2.0 * (1.0 - x);
        gradient[2] = 200.0 * (y - x * x);
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
struct Paraboloid3D;
impl MultipleVarFunction for Paraboloid3D {
    fn variables(&self) -> i32 {
        3
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        if point.length() != 3 {
            return false;
        }
        *value = point[1] * point[1] + 2.0 * point[2] * point[2] + 3.0 * point[3] * point[3];
        true
    }
}
impl MultipleVarFunctionWithGradient for Paraboloid3D {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        if point.length() != 3 || gradient.length() != 3 {
            return false;
        }
        gradient[1] = 2.0 * point[1];
        gradient[2] = 4.0 * point[2];
        gradient[3] = 6.0 * point[3];
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
struct SquareFunction1D;
impl MultipleVarFunction for SquareFunction1D {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        if point.length() != 1 {
            return false;
        }
        let x: f64 = point[1];
        *value = x * x;
        true
    }
}
impl MultipleVarFunctionWithGradient for SquareFunction1D {
    fn gradient(&mut self, point: &Vector<'_>, gradient: &mut Vector<'_>) -> bool {
        if point.length() != 1 || gradient.length() != 1 {
            return false;
        }
        let x: f64 = point[1];
        gradient[1] = 2.0 * x;
        true
    }
    fn values(&mut self, point: &Vector<'_>, value: &mut f64, gradient: &mut Vector<'_>) -> bool {
        self.value(point, value) && self.gradient(point, gradient)
    }
}
include!("math_bfgs/suites_0.rs");
include!("math_bfgs/suites_1.rs");
include!("math_bfgs/suites_2.rs");
include!("math_bfgs/suites_3.rs");
include!("math_bfgs/suites_4.rs");
