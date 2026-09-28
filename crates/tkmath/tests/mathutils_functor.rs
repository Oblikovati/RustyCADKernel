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
