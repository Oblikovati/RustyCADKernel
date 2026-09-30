//! Direct MathSys_Comparison_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, unused_mut, unused_variables)]
use rustycad_tkmath::{
    math::{FunctionSet, FunctionSetWithDerivatives, Matrix, NewtonFunctionSetRoot, Vector},
    math_sys::*,
    math_utils::Status,
};
const THE_TOLERANCE: f64 = 1e-8;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
include!("mathsys_comparison/fixture_SimpleCircleLineSystem.rs");
include!("mathsys_comparison/fixture_RosenbrockSystem.rs");
include!("mathsys_comparison/fixture_DiagonalQuadraticSystem.rs");
include!("mathsys_comparison/fixture_BoundaryTangentSystem.rs");
include!("mathsys_comparison/fixture_Linear3System.rs");
include!("mathsys_comparison/fixture_PolynomialSystem.rs");
include!("mathsys_comparison/fixture_RectangularSystem.rs");
include!("mathsys_comparison/fixture_SingularSystem.rs");
include!("mathsys_comparison/fixture_FailingSystem.rs");
include!("mathsys_comparison/fixture_CallbackFailingSystem.rs");
include!("mathsys_comparison/fixture_CoupledBoundedSystem.rs");
include!("mathsys_comparison/fixture_ExponentialCircleFunctionSet.rs");
include!("mathsys_comparison/fixture_TrigonometricProductFunctionSet.rs");
include!("mathsys_comparison/fixture_MixedSpaceFunctionSet.rs");
include!("mathsys_comparison/fixture_CubicSpaceFunctionSet.rs");
include!("mathsys_comparison/fixture_OverdeterminedTranscendentalFunctionSet.rs");
include!("mathsys_comparison/fixture_BoundedLogarithmicFunctionSet.rs");
include!("mathsys_comparison/tests_0.rs");
include!("mathsys_comparison/tests_1.rs");
include!("mathsys_comparison/tests_2.rs");
include!("mathsys_comparison/tests_3.rs");
include!("mathsys_comparison/tests_4.rs");
include!("mathsys_comparison/tests_5.rs");
include!("mathsys_comparison/tests_6.rs");
include!("mathsys_comparison/tests_7.rs");
include!("mathsys_comparison/tests_8.rs");
include!("mathsys_comparison/tests_9.rs");
include!("mathsys_comparison/tests_10.rs");
include!("mathsys_comparison/tests_11.rs");
