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

//! MathLin_Comparison_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case, unused_mut)]
use rustycad_tkmath::math::{Gauss, Matrix, Vector};
use rustycad_tkmath::math_lin::*;
const THE_TOLERANCE: f64 = 1e-10;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
include!("mathlin_comparison/tests_0.rs");
include!("mathlin_comparison/tests_1.rs");
include!("mathlin_comparison/tests_2.rs");
include!("mathlin_comparison/tests_3.rs");
