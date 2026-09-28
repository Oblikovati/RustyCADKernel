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

//! Direct MathOpt_GlobOpt_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    clippy::field_reassign_with_default
)]
use rustycad_tkmath::{
    math::{GlobalObjective, GlobalOptimum, MultipleVarFunction, Vector},
    math_opt::*,
    math_utils::{Status, VectorResult},
};
const THE_TOLERANCE: f64 = 1e-6;
const THE_STOCHASTIC_TOLERANCE: f64 = 1e-2;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
include!("mathopt_global/fixture_SphereFunc.rs");
include!("mathopt_global/fixture_RastriginFunc.rs");
include!("mathopt_global/fixture_RosenbrockFunc.rs");
include!("mathopt_global/fixture_BoothFunc.rs");
include!("mathopt_global/fixture_OutsideMinimumFunc.rs");
include!("mathopt_global/fixture_PartiallyDefinedFunc.rs");
include!("mathopt_global/fixture_RejectAllFunc.rs");
include!("mathopt_global/fixture_HimmelblauFunc.rs");
include!("mathopt_global/fixture_GoldsteinPriceFunc.rs");
include!("mathopt_global/fixture_SphereFuncOld.rs");
include!("mathopt_global/test_0.rs");
include!("mathopt_global/test_1.rs");
include!("mathopt_global/test_2.rs");
include!("mathopt_global/test_3.rs");
include!("mathopt_global/test_4.rs");
include!("mathopt_global/test_5.rs");
include!("mathopt_global/test_6.rs");
include!("mathopt_global/test_7.rs");
include!("mathopt_global/test_8.rs");
include!("mathopt_global/test_9.rs");
include!("mathopt_global/test_10.rs");
include!("mathopt_global/test_11.rs");
include!("mathopt_global/test_12.rs");
include!("mathopt_global/test_13.rs");
include!("mathopt_global/test_14.rs");
include!("mathopt_global/test_15.rs");
include!("mathopt_global/test_16.rs");
include!("mathopt_global/test_17.rs");
include!("mathopt_global/test_18.rs");
include!("mathopt_global/test_19.rs");
include!("mathopt_global/test_20.rs");
include!("mathopt_global/test_21.rs");
include!("mathopt_global/test_22.rs");
include!("mathopt_global/test_23.rs");
