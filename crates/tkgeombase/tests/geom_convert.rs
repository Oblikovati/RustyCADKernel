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

//! Direct translation of GeomConvert_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomCircle, GeomCurve, GeomLine, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkgeombase::geom_convert as GeomConvert;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{self, Axis2, Direction3, Point3};

#[test]
fn GeomConvertTest__CircleToBSpline() {
    let aCircle: Handle<GeomCircle> = Handle::new(
        GeomCircle::new(
            Axis2::new(
                Point3::new(0.0, 0.0, 0.0),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
            )
            .unwrap(),
            5.0,
        )
        .unwrap(),
    );

    let aBSpline: Handle<GeomBSplineCurve> =
        GeomConvert::curve_to_bspline_curve(&aCircle.cast(), ParameterisationType::TgtThetaOver2)
            .unwrap();

    assert!(!aBSpline.is_null());

    let aOrigPnt: Point3 = aCircle.d0(0.0).unwrap();
    let aBSplinePnt: Point3 = aBSpline.d0(0.0).unwrap();

    EXPECT_NEAR(aOrigPnt.distance(&aBSplinePnt), 0.0, precision::confusion());
}

#[test]
fn GeomConvertTest__LineToBSpline() {
    let aLine: Handle<GeomLine> = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    ));
    let aTrimmed: Handle<GeomTrimmedCurve> =
        Handle::new(GeomTrimmedCurve::new(&aLine.cast(), 0.0, 10.0, true, true).unwrap());

    let aBSpline: Handle<GeomBSplineCurve> = GeomConvert::curve_to_bspline_curve(
        &aTrimmed.cast::<dyn GeomCurve>(),
        ParameterisationType::TgtThetaOver2,
    )
    .unwrap();

    assert!(!aBSpline.is_null());

    let aPnt: Point3 = aBSpline.d0(5.0).unwrap();
    let anExpectedPnt = Point3::new(5.0, 0.0, 0.0);

    EXPECT_NEAR(aPnt.distance(&anExpectedPnt), 0.0, precision::confusion());
}

#[test]
fn GeomConvertTest__PlaneToBSplineSurface() {
    let aPlane: Handle<GeomPlane> = Handle::new(GeomPlane::new(gp::Axis3::from_axis2(gp::xoy())));
    let aTrimmed: Handle<GeomRectangularTrimmedSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(&aPlane.cast(), (0.0, 10.0), (0.0, 10.0), (true, true))
            .unwrap(),
    );

    let aBSurf: Handle<GeomBSplineSurface> =
        GeomConvert::surface_to_bspline_surface(&aTrimmed.cast::<dyn GeomSurface>()).unwrap();

    assert!(!aBSurf.is_null());

    let aPnt: Point3 = aBSurf.d0(5.0, 5.0).unwrap();
    let anExpectedPnt = Point3::new(5.0, 5.0, 0.0);

    EXPECT_NEAR(aPnt.distance(&anExpectedPnt), 0.0, precision::confusion());
}
