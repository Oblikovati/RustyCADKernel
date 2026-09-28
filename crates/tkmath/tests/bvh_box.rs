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

//! Direct BVH_Box_Test.cxx translation at pinned OCCT 3d097a0.
#![allow(non_snake_case, non_upper_case_globals, unused_mut)]
use rustycad_tkernel::ncollection::{Mat4, Vec3};
use rustycad_tkernel::precision;
use rustycad_tkmath::bvh::{BvhBox, BvhVec2d, BvhVec3d, BvhVec3f, BvhVec4d};

fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!(
        (left - right).abs() <= tolerance,
        "{left} != {right} within {tolerance}"
    );
}

/// GoogleTest promotes float `EXPECT_NEAR` arguments to double.
fn assert_near_f32(left: f32, right: f32, tolerance: f32) {
    assert_near(f64::from(left), f64::from(right), f64::from(tolerance));
}

include!("bvh_box/suites_0.rs");
include!("bvh_box/suites_1.rs");
