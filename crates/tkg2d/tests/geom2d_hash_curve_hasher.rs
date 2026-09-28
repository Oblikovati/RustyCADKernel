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

//! Direct translation of Geom2dHash_CurveHasher_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dGeometry, Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola,
    Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_hash::Geom2dHashCurveHasher;
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2, Vector2};

// The fixture member `Geom2dHash_CurveHasher myHasher;` with the default tolerances.
fn SetUp() -> Geom2dHashCurveHasher {
    Geom2dHashCurveHasher::default()
}

// The implicit `occ::handle<T>` to `occ::handle<Geom2d_Curve>` conversion at each call.
fn curve<T: ?Sized + RuntimeTyped>(handle: &Handle<T>) -> Handle<dyn Geom2dCurve> {
    handle.cast()
}

// `occ::down_cast<T>(theCurve->Copy())`.
fn copied<T: RuntimeTyped + Geom2dGeometry>(handle: &Handle<T>) -> Handle<T> {
    handle.copy().unwrap().cast()
}

fn pnt(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}
fn dir(x: f64, y: f64) -> Direction2 {
    Direction2::new(x, y).unwrap()
}
fn axes(x_dir: Direction2, y_dir: Direction2) -> Axis22d {
    Axis22d::with_directions(pnt(0.0, 0.0), x_dir, y_dir).unwrap()
}
fn standard_axes() -> Axis22d {
    axes(dir(1.0, 0.0), dir(0.0, 1.0))
}
fn line(location: Point2, direction: Direction2) -> Handle<Geom2dLine> {
    Handle::new(Geom2dLine::from_point(location, direction))
}
fn circle(frame: Axis22d, radius: f64) -> Handle<Geom2dCircle> {
    Handle::new(Geom2dCircle::new(frame, radius).unwrap())
}

fn poles(points: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut aPoles: Array1<'static, Point2> = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, &(x, y)) in points.iter().enumerate() {
        aPoles[i as i32 + 1] = pnt(x, y);
    }
    aPoles
}
fn reals(values: &[f64]) -> Array1<'static, f64> {
    let mut anArray: Array1<'static, f64> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, &value) in values.iter().enumerate() {
        anArray[i as i32 + 1] = value;
    }
    anArray
}
fn integers(values: &[i32]) -> Array1<'static, i32> {
    let mut anArray: Array1<'static, i32> = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, &value) in values.iter().enumerate() {
        anArray[i as i32 + 1] = value;
    }
    anArray
}

include!("geom2d_hash_curve_hasher/suites_0.rs");
include!("geom2d_hash_curve_hasher/suites_1.rs");
