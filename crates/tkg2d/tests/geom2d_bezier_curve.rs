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

//! Direct translation of Geom2d_BezierCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};

// The fixture SetUp: a cubic Bezier curve through (0,0), (1,1), (2,1), (3,0).
fn SetUp() -> Handle<Geom2dBezierCurve> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point2::new(0.0, 0.0);
    aPoles[2] = Point2::new(1.0, 1.0);
    aPoles[3] = Point2::new(2.0, 1.0);
    aPoles[4] = Point2::new(3.0, 0.0);
    Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap())
}

fn poles3(p1: Point2, p2: Point2, p3: Point2) -> Array1<'static, Point2> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, 3).unwrap();
    aPoles[1] = p1;
    aPoles[2] = p2;
    aPoles[3] = p3;
    aPoles
}

fn weights3(w1: f64, w2: f64, w3: f64) -> Array1<'static, f64> {
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    aWeights[1] = w1;
    aWeights[2] = w2;
    aWeights[3] = w3;
    aWeights
}

include!("geom2d_bezier_curve/suites_0.rs");
include!("geom2d_bezier_curve/suites_1.rs");
