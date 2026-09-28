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

//! Direct translation of Geom_BSplineCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve, GeomGeometry};
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};

fn pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

// A one-based NCollection_Array1 filled with `values`.
fn array<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut result: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

// The fixture SetUp: a cubic Bezier-like B-spline through (0,0,0), (1,1,0), (2,1,0), (3,0,0).
fn SetUp() -> Handle<GeomBSplineCurve> {
    let aPoles = array(&[
        pnt(0.0, 0.0, 0.0),
        pnt(1.0, 1.0, 0.0),
        pnt(2.0, 1.0, 0.0),
        pnt(3.0, 0.0, 0.0),
    ]);
    let aKnots = array(&[0.0, 1.0]);
    let aMults = array(&[4, 4]);
    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap())
}

// The quadratic rational curve of weights (1, w2, 1) on knots {0, 1} x 3 used by several tests.
fn rational_quadratic(p: [Point3; 3], w2: f64) -> Handle<GeomBSplineCurve> {
    let aPoles = array(&p);
    let aWeights = array(&[1.0, w2, 1.0]);
    let aKnots = array(&[0.0, 1.0]);
    let aMults = array(&[3, 3]);
    Handle::new(
        GeomBSplineCurve::with_weights(&aPoles, &aWeights, &aKnots, &aMults, 2, false).unwrap(),
    )
}

// The periodic cubic of PeriodicCurve and PeriodicCurve_SetOrigin.
fn periodic_pentagon() -> Handle<GeomBSplineCurve> {
    let aPoles = array(&[
        pnt(1.0, 0.0, 0.0),
        pnt(0.309, 0.951, 0.0),
        pnt(-0.809, 0.588, 0.0),
        pnt(-0.809, -0.588, 0.0),
        pnt(0.309, -0.951, 0.0),
    ]);
    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 6).unwrap();
    for i in 1..=6 {
        aKnots[i] = f64::from(i - 1) * 0.2;
    }
    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 6).unwrap();
    aMults.init(1);
    Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, true).unwrap())
}

// The `new Geom_BSplineCurve(*curve)` of the copy tests.
fn copied(curve: &GeomBSplineCurve) -> Handle<GeomBSplineCurve> {
    Handle::new(GeomBSplineCurve::copy_of(curve).unwrap())
}

include!("geom_bspline_curve/suites_0.rs");
include!("geom_bspline_curve/suites_1.rs");
