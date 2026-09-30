//! Direct math_FunctionRoot_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case)]
use rustycad_tkmath::math::{Function, FunctionRoot, FunctionWithDerivative};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct QuadraticFunction;
impl Function for QuadraticFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x * x - 4.0;
        true
    }
}
impl FunctionWithDerivative for QuadraticFunction {
    fn derivative(&mut self, x: f64, result: &mut f64) -> bool {
        *result = 2.0 * x;
        true
    }
    fn values(&mut self, x: f64, result: &mut f64, derivative: &mut f64) -> bool {
        *result = x * x - 4.0;
        *derivative = 2.0 * x;
        true
    }
}
struct CubicFunction;
impl Function for CubicFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x * x * x - 6.0 * x * x + 11.0 * x - 6.0;
        true
    }
}
impl FunctionWithDerivative for CubicFunction {
    fn derivative(&mut self, x: f64, result: &mut f64) -> bool {
        *result = 3.0 * x * x - 12.0 * x + 11.0;
        true
    }
    fn values(&mut self, x: f64, result: &mut f64, derivative: &mut f64) -> bool {
        *result = x * x * x - 6.0 * x * x + 11.0 * x - 6.0;
        *derivative = 3.0 * x * x - 12.0 * x + 11.0;
        true
    }
}
struct SinFunction;
impl Function for SinFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x.sin();
        true
    }
}
impl FunctionWithDerivative for SinFunction {
    fn derivative(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x.cos();
        true
    }
    fn values(&mut self, x: f64, result: &mut f64, derivative: &mut f64) -> bool {
        *result = x.sin();
        *derivative = x.cos();
        true
    }
}
struct ZeroDerivativeFunction;
impl Function for ZeroDerivativeFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x * x;
        true
    }
}
impl FunctionWithDerivative for ZeroDerivativeFunction {
    fn derivative(&mut self, x: f64, result: &mut f64) -> bool {
        *result = 2.0 * x;
        true
    }
    fn values(&mut self, x: f64, result: &mut f64, derivative: &mut f64) -> bool {
        *result = x * x;
        *derivative = 2.0 * x;
        true
    }
}
include!("math_function_root/suites_0.rs");
include!("math_function_root/suites_1.rs");
include!("math_function_root/suites_2.rs");
include!("math_function_root/suites_3.rs");
include!("math_function_root/suites_4.rs");
