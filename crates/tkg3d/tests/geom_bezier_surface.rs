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

//! Direct translation of Geom_BezierSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomCurve, GeomGeometry, GeomSurface};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

fn pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

// The fixture SetUp: a 3x3 Bezier surface with poles (i, j, (i + j) * 0.1).
fn SetUp() -> Handle<GeomBezierSurface> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for i in 1..=3 {
        for j in 1..=3 {
            let (x, y) = (f64::from(i), f64::from(j));
            aPoles[(i, j)] = pnt(x, y, (x + y) * 0.1);
        }
    }
    Handle::new(GeomBezierSurface::new(&aPoles).unwrap())
}

// The 2x2 grid (0,0,0) (1,0,0) / (0,1,0) (1,1,z22) that many tests build.
fn unit_poles(z22: f64) -> Array2<'static, Point3> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aPoles[(1, 1)] = pnt(0.0, 0.0, 0.0);
    aPoles[(1, 2)] = pnt(1.0, 0.0, 0.0);
    aPoles[(2, 1)] = pnt(0.0, 1.0, 0.0);
    aPoles[(2, 2)] = pnt(1.0, 1.0, z22);
    aPoles
}

// A 2x2 weight grid in the native assignment order.
fn weights22(w11: f64, w12: f64, w21: f64, w22: f64) -> Array2<'static, f64> {
    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aWeights[(1, 1)] = w11;
    aWeights[(1, 2)] = w12;
    aWeights[(2, 1)] = w21;
    aWeights[(2, 2)] = w22;
    aWeights
}

// A one-based line of points.
fn line(points: &[Point3]) -> Array1<'static, Point3> {
    let mut aLine: Array1<'static, Point3> = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (k, p) in points.iter().enumerate() {
        aLine[k as i32 + 1] = *p;
    }
    aLine
}

// A one-based line of weights.
fn line_weights(values: &[f64]) -> Array1<'static, f64> {
    let mut aLine: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (k, w) in values.iter().enumerate() {
        aLine[k as i32 + 1] = *w;
    }
    aLine
}

// `new Geom_BezierSurface(*surface)`.
fn copied(surface: &GeomBezierSurface) -> Handle<GeomBezierSurface> {
    Handle::new(GeomBezierSurface::copy_of(surface).unwrap())
}

include!("geom_bezier_surface/suites_0.rs");
include!("geom_bezier_surface/suites_1.rs");
include!("geom_bezier_surface/suites_2.rs");
