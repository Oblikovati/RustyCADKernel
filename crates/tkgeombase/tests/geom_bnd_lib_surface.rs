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

//! Direct translation of GeomBndLib_Surface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! The suites are split across `geom_bnd_lib_surface/suites_N.rs` in the upstream order.
#![allow(non_snake_case)]
mod bnd_support;
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use bnd_support::{array1, array2, dir, get3, pnt, surface};
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::bnd_lib::AddSurface;
use rustycad_tkgeombase::geom_bnd_lib::Surface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{self, Axis3, Cone3};
use std::f64::consts::PI;

// Helper: compare two Bnd_Box results within tolerance.
fn CompareBoxes(theNew: &BoundingBox3, theOld: &BoundingBox3, theTol: f64) {
    for (aN, aO) in get3(theNew).iter().zip(get3(theOld)) {
        EXPECT_NEAR(*aN, aO, theTol);
    }
}

// The C++ tests' `ExpectNoLarger` blocks: every side of `theNew` inside `theOld`.
fn expect_no_larger(theNew: &BoundingBox3, theOld: &BoundingBox3) {
    let [aXminN, aXmaxN, aYminN, aYmaxN, aZminN, aZmaxN] = get3(theNew);
    let [aXminO, aXmaxO, aYminO, aYmaxO, aZminO, aZmaxO] = get3(theOld);
    let aTol = precision::confusion();
    assert!(aXminN >= aXminO - aTol);
    assert!(aXmaxN <= aXmaxO + aTol);
    assert!(aYminN >= aYminO - aTol);
    assert!(aYmaxN <= aYmaxO + aTol);
    assert!(aZminN >= aZminO - aTol);
    assert!(aZmaxN <= aZmaxO + aTol);
}

fn xoy() -> Axis3 {
    gp::xoy().axis3()
}

fn adaptor(theSurf: &Handle<dyn GeomSurface>) -> GeomAdaptorSurface {
    GeomAdaptorSurface::with_surface(theSurf).unwrap()
}

// GeomBndLib_Surface(handle).Add over the patch, or the natural bounds when None.
fn new_box(theSurf: &Handle<dyn GeomSurface>, thePatch: Option<[f64; 4]>) -> BoundingBox3 {
    let anEval = Surface::from_surface(theSurf).unwrap();
    let mut aBox = BoundingBox3::default();
    match thePatch {
        Some(aPatch) => anEval
            .add_range(aPatch, precision::confusion(), &mut aBox)
            .unwrap(),
        None => anEval.add(precision::confusion(), &mut aBox).unwrap(),
    }
    aBox
}

// BndLib_AddSurface::Add on a GeomAdaptor_Surface over the patch or the adaptor bounds.
fn old_box(theAdaptor: &dyn Adaptor3dSurface, thePatch: Option<[f64; 4]>) -> BoundingBox3 {
    let mut aBox = BoundingBox3::default();
    match thePatch {
        Some(aPatch) => {
            AddSurface::add_range(theAdaptor, aPatch, precision::confusion(), &mut aBox).unwrap()
        }
        None => AddSurface::add(theAdaptor, precision::confusion(), &mut aBox).unwrap(),
    }
    aBox
}

fn sphere(theRadius: f64) -> Handle<dyn GeomSurface> {
    surface(GeomSphericalSurface::new(xoy(), theRadius).unwrap())
}

fn torus() -> Handle<dyn GeomSurface> {
    surface(GeomToroidalSurface::new(xoy(), 10.0, 3.0).unwrap())
}

include!("geom_bnd_lib_surface/suites_0.rs");
include!("geom_bnd_lib_surface/suites_1.rs");
