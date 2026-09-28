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

//! Direct translation of Geom_BSplineSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomCurve, GeomGeometry, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::bsplslib::MoveRange;
use rustycad_tkmath::gp::{Point3, Transform3, Vector3};
use std::f64::consts::PI;

fn pnt(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

// A one-based NCollection_Array1 filled with `values`.
fn array<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut result: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

// A one-based NCollection_Array2 of `nu` x `nv` cells, cell (i, j) = `cell(i, j)`.
fn grid<T: Copy + Default>(nu: i32, nv: i32, cell: impl Fn(i32, i32) -> T) -> Array2<'static, T> {
    let mut result: Array2<'static, T> = Array2::with_bounds(1, nu, 1, nv).unwrap();
    for i in 1..=nu {
        for j in 1..=nv {
            result.set_value(i, j, cell(i, j)).unwrap();
        }
    }
    result
}

// The fixture SetUp: a biquadratic Bezier-like surface, poles (i, j, (i + j) * 0.1).
fn SetUp() -> Handle<GeomBSplineSurface> {
    // Create a simple BSpline surface for testing
    let aPoles = grid(3, 3, |i, j| {
        pnt(f64::from(i), f64::from(j), f64::from(i + j) * 0.1)
    });
    let anUKnots = array(&[0.0, 1.0]);
    let aVKnots = array(&[0.0, 1.0]);
    let anUMults = array(&[3, 3]);
    let aVMults = array(&[3, 3]);
    Handle::new(
        GeomBSplineSurface::new(
            &aPoles,
            (&anUKnots, &anUMults),
            (&aVKnots, &aVMults),
            (2, 2),
            (false, false),
        )
        .unwrap(),
    )
}

// `new Geom_BSplineSurface(aPoles, aWeights, {0, 1}, {0, 1}, {3, 3}, {3, 3}, 2, 2)`, the
// rational biquadratic patch that several tests build.
fn rational_patch(
    aPoles: &Array2<'static, Point3>,
    aWeights: &Array2<'static, f64>,
) -> Handle<GeomBSplineSurface> {
    let anUKnots = array(&[0.0, 1.0]);
    let aVKnots = array(&[0.0, 1.0]);
    let anUMults = array(&[3, 3]);
    let aVMults = array(&[3, 3]);
    Handle::new(
        GeomBSplineSurface::with_weights(
            aPoles,
            aWeights,
            (&anUKnots, &anUMults),
            (&aVKnots, &aVMults),
            (2, 2),
            (false, false),
        )
        .unwrap(),
    )
}

// The poles of SetWeight, SetWeightRow and SetWeightCol: the unit grid with the centre
// pole raised to z = 1, and unit weights but 2 at the centre.
fn raised_centre() -> (Array2<'static, Point3>, Array2<'static, f64>) {
    let aPoles = grid(3, 3, |i, j| {
        pnt(
            f64::from(i),
            f64::from(j),
            if i == 2 && j == 2 { 1.0 } else { 0.0 },
        )
    });
    let aWeights = grid(3, 3, |i, j| if i == 2 && j == 2 { 2.0 } else { 1.0 });
    (aPoles, aWeights)
}

// The `new Geom_BSplineSurface(*surface)` of the copy tests.
fn copied(surface: &GeomBSplineSurface) -> Handle<GeomBSplineSurface> {
    Handle::new(GeomBSplineSurface::copy_of(surface).unwrap())
}

// One knot vector: knots and multiplicities.
type Knots = (Array1<'static, f64>, Array1<'static, i32>);

// The degree 2 periodic direction of 4 knots of multiplicity 1 (3 poles) and the
// clamped direction {0, 1} x 3.
fn periodic_knots() -> (Knots, Knots) {
    let aPeriodic = (array(&[0.0, 1.0, 2.0, 3.0]), array(&[1, 1, 1, 1]));
    let aClamped = (array(&[0.0, 1.0]), array(&[3, 3]));
    (aPeriodic, aClamped)
}

// The angle 2 * PI * (k - 1) / 3 of the periodic tests.
fn third(k: i32) -> f64 {
    2.0 * PI * f64::from(k - 1) / 3.0
}

include!("geom_bspline_surface/suites_0.rs");
include!("geom_bspline_surface/suites_1.rs");
