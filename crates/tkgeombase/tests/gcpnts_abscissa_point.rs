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

//! Direct translation of GCPnts_AbscissaPoint_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::prelude::*;
use rustycad_tkgeombase::gcpnts::AbscissaPoint as GCPnts_AbscissaPoint;

// `GeomAdaptor_Curve(aLine, 0.0, 10.0)` on the X axis.
fn lineAdaptor() -> GeomAdaptorCurve {
    let aLine: Handle<GeomLine> = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        dir3(1.0, 0.0, 0.0),
    ));
    GeomAdaptorCurve::with_range(&aLine.cast(), 0.0, 10.0).unwrap()
}

// `GeomAdaptor_Curve(aCircle, 0.0, theLast)` on a circle of the XY plane.
fn circleAdaptor(theRadius: f64, theLast: f64) -> GeomAdaptorCurve {
    let aCircle: Handle<GeomCircle> = Handle::new(
        GeomCircle::new(
            Axis2::new(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)).unwrap(),
            theRadius,
        )
        .unwrap(),
    );
    GeomAdaptorCurve::with_range(&aCircle.cast(), 0.0, theLast).unwrap()
}

#[test]
fn GCPnts_AbscissaPointTest__LineLength() {
    let anAdaptor = lineAdaptor();

    let aLength: f64 = GCPnts_AbscissaPoint::length(&anAdaptor).unwrap();

    EXPECT_NEAR(aLength, 10.0, precision::confusion());
}

#[test]
fn GCPnts_AbscissaPointTest__CircleArcLength() {
    let anAdaptor = circleAdaptor(1.0, PI);

    let aLength: f64 = GCPnts_AbscissaPoint::length(&anAdaptor).unwrap();

    EXPECT_NEAR(aLength, PI, precision::confusion());
}

#[test]
fn GCPnts_AbscissaPointTest__FullCircleLength() {
    let anAdaptor = circleAdaptor(5.0, 2.0 * PI);

    let aLength: f64 = GCPnts_AbscissaPoint::length(&anAdaptor).unwrap();

    EXPECT_NEAR(aLength, 2.0 * PI * 5.0, precision::confusion());
}

#[test]
fn GCPnts_AbscissaPointTest__ParameterAtAbscissa_Line() {
    let anAdaptor = lineAdaptor();

    let anAbscissa = GCPnts_AbscissaPoint::from_abscissa(&anAdaptor, 5.0, 0.0).unwrap();

    assert!(anAbscissa.is_done());
    let aParam: f64 = anAbscissa.parameter().unwrap();
    EXPECT_NEAR(aParam, 5.0, precision::confusion());
}

#[test]
fn GCPnts_AbscissaPointTest__ParameterAtAbscissa_Circle() {
    let anAdaptor = circleAdaptor(1.0, 2.0 * PI);

    let anAbscissa = GCPnts_AbscissaPoint::from_abscissa(&anAdaptor, PI / 2.0, 0.0).unwrap();

    assert!(anAbscissa.is_done());
    let aParam: f64 = anAbscissa.parameter().unwrap();
    EXPECT_NEAR(aParam, PI / 2.0, precision::confusion());
}
