// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Direct MathSys_LM_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    clippy::field_reassign_with_default
)]
use rustycad_tkmath::{
    math::{FunctionSet, FunctionSetRoot, FunctionSetWithDerivatives, Matrix, Vector},
    math_sys::*,
    math_utils::Status,
};
const THE_TOLERANCE: f64 = 1e-6;
const THE_LOOSE_TOL: f64 = 1e-3;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
include!("mathsys_lm/fixture_LinearSystem2D.rs");
include!("mathsys_lm/fixture_CircleHyperbola.rs");
include!("mathsys_lm/fixture_RosenbrockResidual.rs");
include!("mathsys_lm/fixture_PowellSingular.rs");
include!("mathsys_lm/fixture_ExponentialFit.rs");
include!("mathsys_lm/fixture_OverdeterminedSystem.rs");
include!("mathsys_lm/fixture_StationaryNonRoot.rs");
include!("mathsys_lm/fixture_NonFiniteResidual.rs");
include!("mathsys_lm/fixture_ScalarResidual.rs");
include!("mathsys_lm/fixture_CallbackFailure.rs");
include!("mathsys_lm/fixture_DerivativeCallbackFailure.rs");
include!("mathsys_lm/fixture_CircleHyperbolaOld.rs");
include!("mathsys_lm/fixture_RosenbrockResidualOld.rs");
include!("mathsys_lm/tests_0.rs");
include!("mathsys_lm/tests_1.rs");
include!("mathsys_lm/tests_2.rs");
include!("mathsys_lm/tests_3.rs");
include!("mathsys_lm/tests_4.rs");
include!("mathsys_lm/tests_5.rs");
include!("mathsys_lm/tests_6.rs");
include!("mathsys_lm/tests_7.rs");
include!("mathsys_lm/tests_8.rs");
include!("mathsys_lm/tests_9.rs");
include!("mathsys_lm/tests_10.rs");
include!("mathsys_lm/tests_11.rs");
include!("mathsys_lm/tests_12.rs");
include!("mathsys_lm/tests_13.rs");
include!("mathsys_lm/tests_14.rs");
