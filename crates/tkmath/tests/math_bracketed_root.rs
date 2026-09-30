//! Direct math_BracketedRoot_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case)]
use rustycad_tkmath::math::{BracketedRoot, Function};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
struct QuadraticFunction;
impl Function for QuadraticFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = (x - 2.0) * (x - 2.0) - 1.0;
        true
    }
}
struct CubicFunction;
impl Function for CubicFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x * x * x - x - 2.0;
        true
    }
}
struct SineFunction;
impl Function for SineFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = x.sin();
        true
    }
}
struct LinearFunction;
impl Function for LinearFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = 2.0 * x - 4.0;
        true
    }
}
include!("math_bracketed_root/suites_0.rs");
include!("math_bracketed_root/suites_1.rs");
include!("math_bracketed_root/suites_2.rs");
include!("math_bracketed_root/suites_3.rs");
include!("math_bracketed_root/suites_4.rs");
