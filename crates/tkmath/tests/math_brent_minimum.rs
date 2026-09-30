//! Direct math_BrentMinimum_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case)]
use rustycad_tkmath::math::{BrentMinimum, Function};
fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}
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
            let dx: f64 = x - 1.0;
            dx * dx * dx * dx + 2.0
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
struct ShiftedExponentialFunction;
impl Function for ShiftedExponentialFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = (x - 3.0).exp();
        true
    }
}
struct RosenbrockSliceFunction;
impl Function for RosenbrockSliceFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        *result = {
            let dx: f64 = 1.0 - x;
            let dy: f64 = x - x * x;
            dx * dx + 100.0 * dy * dy
        };
        true
    }
}
include!("math_brent_minimum/suites_0.rs");
include!("math_brent_minimum/suites_1.rs");
include!("math_brent_minimum/suites_2.rs");
include!("math_brent_minimum/suites_3.rs");
include!("math_brent_minimum/suites_4.rs");
include!("math_brent_minimum/suites_5.rs");
