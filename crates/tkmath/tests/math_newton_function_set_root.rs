//! Direct math_NewtonFunctionSetRoot_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut, unused_variables)]
use rustycad_tkmath::math::{
    FunctionSet, FunctionSetWithDerivatives, Matrix, NewtonFunctionSetRoot, Vector,
};
const TOLERANCE: f64 = 1e-6;
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct CircleLineSystem;
impl FunctionSet for CircleLineSystem {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        F[1] = X[1] * X[1] + X[2] * X[2] - 1.0; // x^2 + y^2 - 1 = 0
        F[2] = X[1] - X[2]; // x - y = 0
        true
    }
}
impl FunctionSetWithDerivatives for CircleLineSystem {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        D[(1, 1)] = 2.0 * X[1]; // df1/dx = 2x
        D[(1, 2)] = 2.0 * X[2]; // df1/dy = 2y
        D[(2, 1)] = 1.0; // df2/dx = 1
        D[(2, 2)] = -1.0; // df2/dy = -1
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F);
        self.derivatives(X, D);
        true
    }
}
struct LinearSystem;
impl FunctionSet for LinearSystem {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        F[1] = 2.0 * X[1] + X[2] - 3.0; // 2x + y - 3 = 0
        F[2] = X[1] + 2.0 * X[2] - 3.0; // x + 2y - 3 = 0
        true
    }
}
impl FunctionSetWithDerivatives for LinearSystem {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        D[(1, 1)] = 2.0; // df1/dx = 2
        D[(1, 2)] = 1.0; // df1/dy = 1
        D[(2, 1)] = 1.0; // df2/dx = 1
        D[(2, 2)] = 2.0; // df2/dy = 2
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F);
        self.derivatives(X, D);
        true
    }
}
struct QuadraticFunction;
impl FunctionSet for QuadraticFunction {
    fn variables(&self) -> i32 {
        1
    }
    fn equations(&self) -> i32 {
        1
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        F[1] = X[1] * X[1] - 4.0;
        true
    }
}
impl FunctionSetWithDerivatives for QuadraticFunction {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        D[(1, 1)] = 2.0 * X[1];
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F);
        self.derivatives(X, D);
        true
    }
}
struct ThreeVariableSystem;
impl FunctionSet for ThreeVariableSystem {
    fn variables(&self) -> i32 {
        3
    }
    fn equations(&self) -> i32 {
        3
    }
    fn value(&mut self, X: &Vector<'_>, F: &mut Vector<'_>) -> bool {
        F[1] = X[1] + X[2] + X[3] - 6.0; // x + y + z - 6 = 0
        F[2] = X[1] - X[2]; // x - y = 0
        F[3] = 2.0 * X[3] - 4.0; // 2z - 4 = 0
        true
    }
}
impl FunctionSetWithDerivatives for ThreeVariableSystem {
    fn derivatives(&mut self, X: &Vector<'_>, D: &mut Matrix<'_>) -> bool {
        let _ = X;
        D[(1, 1)] = 1.0;
        D[(1, 2)] = 1.0;
        D[(1, 3)] = 1.0;
        D[(2, 1)] = 1.0;
        D[(2, 2)] = -1.0;
        D[(2, 3)] = 0.0;
        D[(3, 1)] = 0.0;
        D[(3, 2)] = 0.0;
        D[(3, 3)] = 2.0;
        true
    }
    fn values(&mut self, X: &Vector<'_>, F: &mut Vector<'_>, D: &mut Matrix<'_>) -> bool {
        self.value(X, F);
        self.derivatives(X, D);
        true
    }
}
include!("math_newton_function_set_root/suites_0.rs");
include!("math_newton_function_set_root/suites_1.rs");
include!("math_newton_function_set_root/suites_2.rs");
include!("math_newton_function_set_root/suites_3.rs");
include!("math_newton_function_set_root/suites_4.rs");
