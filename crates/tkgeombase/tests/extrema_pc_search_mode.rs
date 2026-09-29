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

//! Direct translation of ExtremaPC_SearchMode_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871: the ExtremaPC search modes against the
//! classic Extrema_ExtPC ([`ExtPC`]).
#![allow(non_snake_case, dead_code)]
mod common;
mod extrema_pc_random;
#[allow(unused_imports)]
use common::*;
use extrema_pc_random::random_points;
#[allow(unused_imports)]
use prelude::*;
use rustycad_tkgeombase::extrema::ExtPC;

const THE_TOL: f64 = 1.0e-6;

//==================================================================================================
// Test fixture for SearchMode tests
//==================================================================================================

/// `Extrema_ExtPC(P, C)` with the native default TolF.
fn old_ext_pc<'a>(point: Point3, adaptor: &'a GeomAdaptorCurve) -> ExtPC<'a> {
    ExtPC::with_curve(point, adaptor, 1.0e-10).unwrap()
}

// Gets minimum distance from old Extrema_ExtPC
fn GetOldMinDistance(theOld: &ExtPC) -> f64 {
    if !theOld.is_done() || theOld.nb_ext().unwrap() == 0 {
        return f64::MAX;
    }
    let mut aMinSqDist: f64 = f64::MAX;
    for i in 1..=theOld.nb_ext().unwrap() {
        if theOld.square_distance(i).unwrap() < aMinSqDist {
            aMinSqDist = theOld.square_distance(i).unwrap();
        }
    }
    aMinSqDist.sqrt()
}

// Gets maximum distance from old Extrema_ExtPC
fn GetOldMaxDistance(theOld: &ExtPC) -> f64 {
    if !theOld.is_done() || theOld.nb_ext().unwrap() == 0 {
        return 0.0;
    }
    let mut aMaxSqDist: f64 = 0.0;
    for i in 1..=theOld.nb_ext().unwrap() {
        if theOld.square_distance(i).unwrap() > aMaxSqDist {
            aMaxSqDist = theOld.square_distance(i).unwrap();
        }
    }
    aMaxSqDist.sqrt()
}

// Gets minimum parameter from old Extrema_ExtPC
fn GetOldMinParameter(theOld: &ExtPC) -> f64 {
    if !theOld.is_done() || theOld.nb_ext().unwrap() == 0 {
        return 0.0;
    }
    let mut aMinSqDist: f64 = f64::MAX;
    let mut aMinIdx: i32 = 1;
    for i in 1..=theOld.nb_ext().unwrap() {
        if theOld.square_distance(i).unwrap() < aMinSqDist {
            aMinSqDist = theOld.square_distance(i).unwrap();
            aMinIdx = i;
        }
    }
    theOld.point(aMinIdx).unwrap().parameter()
}

// Gets maximum parameter from old Extrema_ExtPC
fn GetOldMaxParameter(theOld: &ExtPC) -> f64 {
    if !theOld.is_done() || theOld.nb_ext().unwrap() == 0 {
        return 0.0;
    }
    let mut aMaxSqDist: f64 = 0.0;
    let mut aMaxIdx: i32 = 1;
    for i in 1..=theOld.nb_ext().unwrap() {
        if theOld.square_distance(i).unwrap() > aMaxSqDist {
            aMaxSqDist = theOld.square_distance(i).unwrap();
            aMaxIdx = i;
        }
    }
    theOld.point(aMaxIdx).unwrap().parameter()
}

/// `gp_Ax2(P, D)`.
fn ax2(p: Point3, d: Direction3) -> Axis2 {
    Axis2::new(p, d).unwrap()
}

/// The circle of radius 10 in the XY plane.
fn circle10() -> GeomAdaptorCurve {
    let aGeomCircle = Handle::new(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );
    GeomAdaptorCurve::with_curve(&aGeomCircle.cast()).unwrap()
}

/// The quarter of that circle (0 to PI/2).
fn quarter_circle10() -> GeomAdaptorCurve {
    let aGeomCircle = Handle::new(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );
    // Only first quadrant: 0 to PI/2
    GeomAdaptorCurve::with_range(&aGeomCircle.cast(), 0.0, PI / 2.0).unwrap()
}

/// The ellipse 20 x 10 in the XY plane.
fn ellipse20x10() -> GeomAdaptorCurve {
    let aGeomEllipse = Handle::new(
        GeomEllipse::new(
            ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)),
            20.0,
            10.0,
        )
        .unwrap(),
    );
    GeomAdaptorCurve::with_curve(&aGeomEllipse.cast()).unwrap()
}

/// The parabola of focal 5 on [-20, 20].
fn parabola5() -> GeomAdaptorCurve {
    let aGeomParabola = Handle::new(
        GeomParabola::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 5.0).unwrap(),
    );
    GeomAdaptorCurve::with_range(&aGeomParabola.cast(), -20.0, 20.0).unwrap()
}

/// The hyperbola 10 x 5 on [-2, 2].
fn hyperbola10x5() -> GeomAdaptorCurve {
    let aGeomHyperbola = Handle::new(
        GeomHyperbola::new(
            ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)),
            10.0,
            5.0,
        )
        .unwrap(),
    );
    GeomAdaptorCurve::with_range(&aGeomHyperbola.cast(), -2.0, 2.0).unwrap()
}

/// The cubic B-spline (0,0)-(1,2)-(3,2)-(4,0).
fn cubic_bspline() -> GeomAdaptorCurve {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 2.0, 0.0);
    aPoles[3] = Point3::new(3.0, 2.0, 0.0);
    aPoles[4] = Point3::new(4.0, 0.0, 0.0);

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMults[1] = 4;
    aMults[2] = 4;

    let aBSpline = Handle::new(GeomBSplineCurve::new(&aPoles, &aKnots, &aMults, 3, false).unwrap());
    GeomAdaptorCurve::with_curve(&aBSpline.cast()).unwrap()
}

/// The cubic Bezier (0,0)-(1,3)-(3,3)-(4,0).
fn cubic_bezier() -> GeomAdaptorCurve {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 4).unwrap();
    aPoles[1] = Point3::new(0.0, 0.0, 0.0);
    aPoles[2] = Point3::new(1.0, 3.0, 0.0);
    aPoles[3] = Point3::new(3.0, 3.0, 0.0);
    aPoles[4] = Point3::new(4.0, 0.0, 0.0);

    let aBezier = Handle::new(GeomBezierCurve::new(&aPoles).unwrap());
    GeomAdaptorCurve::with_curve(&aBezier.cast()).unwrap()
}

/// The circle of radius 10 offset by 5.
fn offset_circle() -> GeomAdaptorCurve {
    let aBaseCircle = Handle::new(
        GeomCircle::new(ax2(Point3::new(0.0, 0.0, 0.0), dir3(0.0, 0.0, 1.0)), 10.0).unwrap(),
    );
    let anOffsetCurve = Handle::new(
        GeomOffsetCurve::new(&aBaseCircle.cast(), 5.0, dir3(0.0, 0.0, 1.0), false).unwrap(),
    );
    GeomAdaptorCurve::with_curve(&anOffsetCurve.cast()).unwrap()
}

// The parameter and on-curve checks of the extremum `index` of a result.
fn check_extremum_on_curve(
    aResult: &ExtremaResult,
    index: usize,
    anAdaptor: &GeomAdaptorCurve,
    (lower, upper): (f64, f64),
    tol: f64,
) {
    let aExtremum = &aResult[index];
    // Parameter should be within bounds
    EXPECT_GE(aExtremum.parameter, lower - THE_TOL);
    EXPECT_LE(aExtremum.parameter, upper + THE_TOL);
    // Point should be on the curve
    let aCurvePnt: Point3 = anAdaptor.value(aExtremum.parameter).unwrap();
    EXPECT_NEAR(aCurvePnt.distance(&aExtremum.point), 0.0, tol);
}

include!("extrema_pc_search_mode/suites_0.rs");
include!("extrema_pc_search_mode/suites_1.rs");
include!("extrema_pc_search_mode/suites_2.rs");
