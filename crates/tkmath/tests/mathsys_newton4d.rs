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

//! Direct MathSys_Newton4D_Test.cxx translation at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    clippy::field_reassign_with_default,
    clippy::needless_range_loop
)]
use rustycad_tkmath::{math_sys::*, math_utils::Status};
const THE_TOL: f64 = 1e-10;
fn near(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "{a} != {b}");
}
use rustycad_tkmath::gp::{Point3, Vector3};
include!("mathsys_newton4d/fixtures.rs");
include!("mathsys_newton4d/test_0.rs");
include!("mathsys_newton4d/test_1.rs");
include!("mathsys_newton4d/test_2.rs");
include!("mathsys_newton4d/test_3.rs");
include!("mathsys_newton4d/test_4.rs");
include!("mathsys_newton4d/test_5.rs");
include!("mathsys_newton4d/test_6.rs");
include!("mathsys_newton4d/test_7.rs");
