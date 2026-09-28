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

//! Direct translation of GeomBndLib_Curve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! The suites are split across `geom_bnd_lib_curve/suites_N.rs` in the upstream order.
#![allow(non_snake_case)]
mod bnd_support;
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use bnd_support::{array1, curve, dir, get3, pnt};
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomHyperbola, GeomLine,
    GeomParabola, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkgeombase::geom_bnd_lib::{Curve, sampling};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp;
use std::f64::consts::PI;

// Helper: compare two Bnd_Box results within tolerance.
fn CompareBoxes(theNew: &BoundingBox3, theOld: &BoundingBox3, theTol: f64) {
    let aNew = get3(theNew);
    let aOld = get3(theOld);
    for (aN, aO) in aNew.iter().zip(aOld) {
        EXPECT_NEAR(*aN, aO, theTol);
    }
}

// Helper: verify that the new box conservatively encloses the old box.
fn ExpectConservativeEnvelope(theNew: &BoundingBox3, theOld: &BoundingBox3, theTol: f64) {
    let [aXminN, aXmaxN, aYminN, aYmaxN, aZminN, aZmaxN] = get3(theNew);
    let [aXminO, aXmaxO, aYminO, aYmaxO, aZminO, aZmaxO] = get3(theOld);
    assert!(aXminN <= aXminO + theTol);
    assert!(aXmaxN >= aXmaxO - theTol);
    assert!(aYminN <= aYminO + theTol);
    assert!(aYmaxN >= aYmaxO - theTol);
    assert!(aZminN <= aZminO + theTol);
    assert!(aZmaxN >= aZmaxO - theTol);
}

fn adaptor(theCurve: &Handle<dyn GeomCurve>) -> GeomAdaptorCurve {
    GeomAdaptorCurve::with_curve(theCurve).unwrap()
}

fn circle(theRadius: f64) -> Handle<dyn GeomCurve> {
    curve(GeomCircle::new(gp::xoy(), theRadius).unwrap())
}

fn ellipse(theMajR: f64, theMinR: f64) -> Handle<dyn GeomCurve> {
    curve(GeomEllipse::new(gp::xoy(), theMajR, theMinR).unwrap())
}

// BndLib_Add3dCurve::Add(adaptor, U1, U2, Tol, B) on a fresh box.
fn old_box_range(
    theCurve: &Handle<dyn GeomCurve>,
    theU1: f64,
    theU2: f64,
    theTol: f64,
) -> BoundingBox3 {
    let mut aBox = BoundingBox3::default();
    Add3dCurve::add_range(&adaptor(theCurve), theU1, theU2, theTol, &mut aBox).unwrap();
    aBox
}

// BndLib_Add3dCurve::Add(adaptor, Tol, B) on a fresh box.
fn old_box(theAdaptor: &dyn Adaptor3dCurve, theTol: f64) -> BoundingBox3 {
    let mut aBox = BoundingBox3::default();
    Add3dCurve::add(theAdaptor, theTol, &mut aBox).unwrap();
    aBox
}

// GeomBndLib_Curve(handle).Add(Tol, B) on a fresh box.
fn new_box(theCurve: &Handle<dyn GeomCurve>, theTol: f64) -> BoundingBox3 {
    let mut aBox = BoundingBox3::default();
    Curve::from_curve(theCurve)
        .unwrap()
        .add(theTol, &mut aBox)
        .unwrap();
    aBox
}

// GeomBndLib_Curve(handle).Add(U1, U2, Tol, B) on a fresh box.
fn new_box_range(
    theCurve: &Handle<dyn GeomCurve>,
    theU1: f64,
    theU2: f64,
    theTol: f64,
) -> BoundingBox3 {
    let mut aBox = BoundingBox3::default();
    Curve::from_curve(theCurve)
        .unwrap()
        .add_range(theU1, theU2, theTol, &mut aBox)
        .unwrap();
    aBox
}

include!("geom_bnd_lib_curve/suites_0.rs");
include!("geom_bnd_lib_curve/suites_1.rs");
