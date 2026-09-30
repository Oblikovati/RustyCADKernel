//! Direct math_BracketMinimum_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    clippy::assertions_on_constants
)]
use rustycad_tkmath::math::{BracketMinimum, Function};
struct QuadraticFunction;
impl Function for QuadraticFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = (x - 2.0) * (x - 2.0) + 1.0;
        true
    }
}
struct QuarticFunction;
impl Function for QuarticFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = {
            let x2: f64 = x * x;
            let x3: f64 = x2 * x;
            let x4: f64 = x3 * x;
            x4 - 4.0 * x3 + 6.0 * x2 - 4.0 * x + 5.0
        };
        true
    }
}
struct CosineFunction;
impl Function for CosineFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x.cos();
        true
    }
}
struct ExponentialFunction;
impl Function for ExponentialFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x.exp();
        true
    }
}
struct MultipleMinFunction;
impl Function for MultipleMinFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x.sin() + 0.1 * x;
        true
    }
}
include!("math_bracket_minimum/suites_0.rs");
include!("math_bracket_minimum/suites_1.rs");
include!("math_bracket_minimum/suites_2.rs");
include!("math_bracket_minimum/suites_3.rs");
include!("math_bracket_minimum/suites_4.rs");
