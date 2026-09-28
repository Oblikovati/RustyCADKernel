// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Direct translation of ExtremaPC2d_DistanceFunction_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//!
//! A native member returns true and writes its outputs; the Rust one returns them
//! (`ASSERT_TRUE(f.Values(u, v, d))` is `f.values(u).unwrap()`).
#![allow(non_snake_case)]
mod common;
use common::{EXPECT_NEAR, dir2};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeombase::extrema_pc2d::DistanceFunction;
use rustycad_tkmath::gp::{Axis22d, Point2};
use std::f64::consts::FRAC_1_SQRT_2;

fn poles(points: &[(f64, f64)]) -> Array1<'static, Point2> {
    let mut aPoles = Array1::with_bounds(1, points.len() as i32).unwrap();
    for (i, (x, y)) in points.iter().enumerate() {
        aPoles[i as i32 + 1] = Point2::new(*x, *y);
    }
    aPoles
}

#[test]
fn ExtremaPC2d_DistanceFunctionTest__LineValuesAreExact() {
    let aLine: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, 0.0),
        dir2(1.0, 0.0),
    ))
    .cast();
    let anAdaptor = Geom2dAdaptorCurve::with_range(&aLine, -4.0, 7.0).unwrap();
    let aFunction = DistanceFunction::new(&anAdaptor, Point2::new(2.0, 3.0));
    let (aValue, aDerivative) = aFunction.values(5.0).unwrap();
    EXPECT_NEAR(aValue, 3.0, 1.0e-12);
    EXPECT_NEAR(aDerivative, 1.0, 1.0e-12);
    let anAdaptorRef: &dyn Adaptor2dCurve2d = &anAdaptor;
    assert!(std::ptr::addr_eq(aFunction.curve(), anAdaptorRef));
    EXPECT_NEAR(aFunction.first_parameter().unwrap(), -4.0, 1.0e-12);
    EXPECT_NEAR(aFunction.last_parameter().unwrap(), 7.0, 1.0e-12);
}

#[test]
fn ExtremaPC2d_DistanceFunctionTest__CircleNormalizedFormulaIsExact() {
    let aFrame = Axis22d::new(Point2::new(0.0, 0.0), dir2(1.0, 0.0), true).unwrap();
    let aCircle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::new(aFrame, 5.0).unwrap()).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCircle).unwrap();
    let aFunction = DistanceFunction::new(&anAdaptor, Point2::new(3.0, -4.0));
    let aU: f64 = 0.7;
    let (aValue, aDerivative) = aFunction.values(aU).unwrap();
    EXPECT_NEAR(aValue, 3.0 * aU.sin() + 4.0 * aU.cos(), 1.0e-11);
    EXPECT_NEAR(aDerivative, 3.0 * aU.cos() - 4.0 * aU.sin(), 1.0e-11);
}

#[test]
fn ExtremaPC2d_DistanceFunctionTest__DerivativeMatchesCenteredDifference() {
    let aPoles = poles(&[(1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
    let mut aWeights: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    aWeights[1] = 1.0;
    aWeights[2] = FRAC_1_SQRT_2;
    aWeights[3] = 1.0;
    let aCurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::with_weights(&aPoles, &aWeights).unwrap()).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let aFunction = DistanceFunction::new(&anAdaptor, Point2::new(2.0, 0.3));
    let aU: f64 = 0.37;
    let aStep: f64 = 1.0e-6;
    let aLeft: f64 = aFunction.value(aU - aStep).unwrap();
    let aRight: f64 = aFunction.value(aU + aStep).unwrap();
    let aDerivative: f64 = aFunction.derivative(aU).unwrap();
    EXPECT_NEAR(aDerivative, (aRight - aLeft) / (2.0 * aStep), 1.0e-6);
}

#[test]
fn ExtremaPC2d_DistanceFunctionTest__ZeroSpeedReturnsDefinedZeros() {
    let aPoles = poles(&[(0.0, 0.0), (0.0, 0.0), (2.0, 1.0), (3.0, 0.0)]);
    let aCurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&aPoles).unwrap()).cast();
    let anAdaptor = Geom2dAdaptorCurve::with_curve(&aCurve).unwrap();
    let aFunction = DistanceFunction::new(&anAdaptor, Point2::new(1.0, 1.0));
    let (aValue, aDerivative) = aFunction.values(0.0).unwrap();
    assert_eq!(aValue, 0.0);
    assert_eq!(aDerivative, 0.0);
}
