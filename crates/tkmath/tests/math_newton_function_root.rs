//! Direct math_NewtonFunctionRoot_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, unused_mut, unused_variables)]
use rustycad_tkmath::math::{Function, FunctionWithDerivative, NewtonFunctionRoot};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct QuadraticWithDerivative;
impl Function for QuadraticWithDerivative {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = x * x - 4.0;
        true
    }
}
impl FunctionWithDerivative for QuadraticWithDerivative {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = 2.0 * x;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = x * x - 4.0;
        *derivative = 2.0 * x;
        true
    }
}
struct CubicWithDerivative;
impl Function for CubicWithDerivative {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = x * x * x - 6.0 * x * x + 11.0 * x - 6.0;
        true
    }
}
impl FunctionWithDerivative for CubicWithDerivative {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = 3.0 * x * x - 12.0 * x + 11.0;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = x * x * x - 6.0 * x * x + 11.0 * x - 6.0;
        *derivative = 3.0 * x * x - 12.0 * x + 11.0;
        true
    }
}
struct SineWithDerivative;
impl Function for SineWithDerivative {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = x.sin();
        true
    }
}
impl FunctionWithDerivative for SineWithDerivative {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = x.cos();
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = x.sin();
        *derivative = x.cos();
        true
    }
}
struct ExponentialWithDerivative;
impl Function for ExponentialWithDerivative {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = x.exp() - 2.0;
        true
    }
}
impl FunctionWithDerivative for ExponentialWithDerivative {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = x.exp();
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = x.exp() - 2.0;
        *derivative = x.exp();
        true
    }
}
struct CubicWithZeroDerivative;
impl Function for CubicWithZeroDerivative {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = x * x * x;
        true
    }
}
impl FunctionWithDerivative for CubicWithZeroDerivative {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        *derivative = 3.0 * x * x;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = x * x * x;
        *derivative = 3.0 * x * x;
        true
    }
}
struct LinearWithDerivative;
impl Function for LinearWithDerivative {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        *value = 2.0 * x - 4.0;
        true
    }
}
impl FunctionWithDerivative for LinearWithDerivative {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let _ = x;
        *derivative = 2.0;
        true
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        *value = 2.0 * x - 4.0;
        *derivative = 2.0;
        true
    }
}
include!("math_newton_function_root/suites_0.rs");
include!("math_newton_function_root/suites_1.rs");
include!("math_newton_function_root/suites_2.rs");
include!("math_newton_function_root/suites_3.rs");
include!("math_newton_function_root/suites_4.rs");
include!("math_newton_function_root/suites_5.rs");
