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

//! Direct translation of Geom2dAPI_ExtremaCurveCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkgeomalgo::geom2d_api::ExtremaCurveCurve;
use rustycad_tkmath::gp::{Direction2, Line2, Point2};

#[test]
fn Geom2dAPI_ExtremaCurveCurveTest__ModDataBug_27467_IntersectingTrimmedLines() {
    let aLine1: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(Line2::new(
        Point2::new(0.0, 0.0),
        Direction2::new(0.0, -1.0).unwrap(),
    )))
    .cast();
    let aLine2: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_lin2d(Line2::new(
        Point2::new(1.0, -9.5),
        Direction2::new(-1.0, 0.0).unwrap(),
    )))
    .cast();
    let aTrimmedLine1: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&aLine1, 0.0, 23.0, true, true).unwrap()).cast();
    let aTrimmedLine2: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&aLine2, 0.0, 2.0, true, true).unwrap()).cast();

    let (first1, last1) = {
        let c = aTrimmedLine1.get().unwrap();
        (c.first_parameter(), c.last_parameter())
    };
    let (first2, last2) = {
        let c = aTrimmedLine2.get().unwrap();
        (c.first_parameter(), c.last_parameter())
    };
    let anExtrema =
        ExtremaCurveCurve::new(&aTrimmedLine1, &aTrimmedLine2, first1, last1, first2, last2)
            .unwrap();
    assert_eq!(anExtrema.nb_extrema(), 1);

    let (aPoint1, aPoint2) = anExtrema.points(1).unwrap();
    EXPECT_NEAR(anExtrema.distance(1).unwrap(), 0.0, 1.0e-9);
    EXPECT_NEAR(aPoint1.x(), 0.0, 1.0e-9);
    EXPECT_NEAR(aPoint1.y(), -9.5, 1.0e-9);
    EXPECT_NEAR(aPoint2.x(), 0.0, 1.0e-9);
    EXPECT_NEAR(aPoint2.y(), -9.5, 1.0e-9);
}
