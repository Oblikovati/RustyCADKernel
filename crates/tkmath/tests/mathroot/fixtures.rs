use rustycad_tkmath::math::{Function, FunctionWithDerivative};
use rustycad_tkmath::math_root::*;
use rustycad_tkmath::math_utils::{Config, ScalarResult, Status};
const THE_TOLERANCE: f64 = 1e-10;
fn near(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() <= tolerance, "{a} != {b}");
}
