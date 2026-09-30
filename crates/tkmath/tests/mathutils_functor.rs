//! Direct MathUtils_Functor_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, unused_mut, clippy::field_reassign_with_default)]
use rustycad_tkmath::math::{
    Function, FunctionWithDerivative, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient,
    Vector,
};
use rustycad_tkmath::math_opt::{bfgs, powell};
use rustycad_tkmath::math_root::{brent, newton};
use rustycad_tkmath::math_utils::scalar::*;
use rustycad_tkmath::math_utils::vector::*;
use rustycad_tkmath::math_utils::{
    Config, NDimConfig, TWO_PI, make_vector, make_vector_with_gradient,
};
const THE_TOLERANCE: f64 = 1e-10;
fn near(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() <= tolerance, "{a} != {b}");
}
include!("mathutils_functor/tests_0.rs");
include!("mathutils_functor/tests_1.rs");
include!("mathutils_functor/tests_2.rs");
include!("mathutils_functor/tests_3.rs");
include!("mathutils_functor/tests_4.rs");
include!("mathutils_functor/tests_5.rs");
include!("mathutils_functor/tests_6.rs");
