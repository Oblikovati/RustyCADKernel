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

//! Direct translation of GeomAPI_ExtremaCurveSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomGeometry, GeomLine, GeomSphericalSurface, GeomSurface,
    GeomTrimmedCurve,
};
use rustycad_tkgeomalgo::geom_api::ExtremaCurveSurface;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3, Transform3};

fn makeCircle(theCenter: Point3, theNormal: Direction3, theRadius: f64) -> Handle<GeomCircle> {
    Handle::new(GeomCircle::new(Axis2::new(theCenter, theNormal).unwrap(), theRadius).unwrap())
}

fn makeSphere(theRadius: f64) -> Handle<GeomSphericalSurface> {
    let frame = Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap();
    Handle::new(GeomSphericalSurface::new(frame, theRadius).unwrap())
}

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

fn expectCircleSphereExtrema(
    theCenter: Point3,
    theNormal: Direction3,
    theCircleRadius: f64,
    theExpectedDistance: f64,
    theIsParallel: bool,
) {
    let aCircle = makeCircle(theCenter, theNormal, theCircleRadius);
    let aSphere = makeSphere(10.0);

    let aCurveFirst = aCircle.first_parameter();
    let aCurveLast = aCircle.last_parameter();
    let (aSurfaceUFirst, aSurfaceULast, aSurfaceVFirst, aSurfaceVLast) = aSphere.bounds();

    let anExtrema = ExtremaCurveSurface::with_ranges(
        &aCircle.clone().cast::<dyn GeomCurve>(),
        &aSphere.clone().cast::<dyn GeomSurface>(),
        (
            (aCurveFirst, aCurveLast),
            (aSurfaceUFirst, aSurfaceULast),
            (aSurfaceVFirst, aSurfaceVLast),
        ),
    )
    .unwrap();
    assert_eq!(anExtrema.is_parallel().unwrap(), theIsParallel);
    if !theIsParallel {
        assert!(anExtrema.nb_extrema() > 0);
    }
    assert!((anExtrema.lower_distance().unwrap() - theExpectedDistance).abs() <= 1.0e-7);
}

// Migrated from tests/lowalgos/extcs/circ_sph_parallel.  A concentric circle
// and sphere form a parallel extrema configuration with a known minimum.
#[test]
fn GeomAPI_ExtremaCurveSurfaceTest__CircSph_ParallelConcentric() {
    expectCircleSphereExtrema(
        Point3::new(0.0, 0.0, 0.0),
        dir(0.0, 0.0, 1.0),
        4.0,
        6.0,
        true,
    );
}

// Migrated from tests/lowalgos/extcs/circ_sph_inter.  A circle on the sphere
// is an intersection case and its minimum extrema distance is zero.
#[test]
fn GeomAPI_ExtremaCurveSurfaceTest__CircSph_Intersection() {
    expectCircleSphereExtrema(
        Point3::new(0.0, 0.0, 0.0),
        dir(0.0, 0.0, 1.0),
        10.0,
        0.0,
        true,
    );
}

// Migrated from tests/lowalgos/extcs/circ_sph_touch.  The internally tangent
// circle must report a zero lower distance even though the circle is bounded.
#[test]
fn GeomAPI_ExtremaCurveSurfaceTest__CircSph_Touch() {
    expectCircleSphereExtrema(
        Point3::new(8.0, 0.0, 0.0),
        dir(0.0, 0.0, 1.0),
        2.0,
        0.0,
        false,
    );
}

// Migrated from tests/lowalgos/extcs/circ_sph_nointer.  A circle outside the
// sphere has a positive minimum distance.
#[test]
fn GeomAPI_ExtremaCurveSurfaceTest__CircSph_NoIntersection() {
    expectCircleSphereExtrema(
        Point3::new(0.0, 0.0, 12.0),
        dir(0.0, 0.0, 1.0),
        2.0,
        148.0_f64.sqrt() - 10.0,
        true,
    );
}

// Migrated from tests/bugs/modalg_5/bug25368_1.  Extrema between a trimmed
// line and a sphere must retain the intersection point on the curve.
#[test]
fn GeomAPI_ExtremaCurveSurfaceTest__ModalgBug_25368_TrimmedLineSphere() {
    let basis: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(0.0, 0.0, 0.2),
        dir(1.0, 1.0, 0.0),
    ))
    .cast();
    let aLine = Handle::new(GeomTrimmedCurve::new(&basis, 0.0, 10.0, true, true).unwrap());
    let aSphere = makeSphere(5.0);

    let (aUFirst, aULast, aVFirst, aVLast) = aSphere.bounds();
    let anExtrema = ExtremaCurveSurface::with_ranges(
        &aLine.clone().cast::<dyn GeomCurve>(),
        &aSphere.clone().cast::<dyn GeomSurface>(),
        (
            (aLine.first_parameter(), aLine.last_parameter()),
            (aUFirst, aULast),
            (aVFirst, aVLast),
        ),
    )
    .unwrap();

    assert_eq!(anExtrema.nb_extrema(), 3);
    let (aCurvePoint, aSurfacePoint) = anExtrema.points(1).unwrap();
    let anExpectedPoint = Point3::new(3.5327043465311383, 3.5327043465311383, 0.2);
    assert!(aCurvePoint.is_equal(&anExpectedPoint, 0.01));
    assert!(aSurfacePoint.is_equal(&anExpectedPoint, 0.01));
    assert!(anExtrema.distance(2).unwrap() > 0.01);
    assert!(anExtrema.distance(3).unwrap() > 0.01);
}

// Migrated from tests/bugs/modalg_5/bug25368_2.  The rotated sphere and
// infinite line have two extrema, both of which are intersection points.
#[test]
fn GeomAPI_ExtremaCurveSurfaceTest__ModalgBug_25368_RotatedSphereLine() {
    let aSphere = makeSphere(4.5);
    let mut aRotation = Transform3::default();
    aRotation
        .set_rotation(
            Axis1::new(Point3::new(0.0, 0.5, 1.0), dir(1.0, 1.0, 0.5)),
            34.9 * std::f64::consts::PI / 180.0,
        )
        .unwrap();
    aSphere.transform(aRotation).unwrap();

    let aLine = Handle::new(GeomLine::from_point(
        Point3::new(-0.79, 0.88, 1.22),
        dir(0.579, 1.765, 0.576),
    ));
    let (aUFirst, aULast, aVFirst, aVLast) = aSphere.bounds();
    let anExtrema = ExtremaCurveSurface::with_ranges(
        &aLine.clone().cast::<dyn GeomCurve>(),
        &aSphere.clone().cast::<dyn GeomSurface>(),
        (
            (aLine.first_parameter(), aLine.last_parameter()),
            (aUFirst, aULast),
            (aVFirst, aVLast),
        ),
    )
    .unwrap();

    assert_eq!(anExtrema.nb_extrema(), 4);
    let anExpectedPoint1 = Point3::new(
        -2.2838350838816694,
        -3.6737459810900646,
        -0.2660950057268425,
    );
    let anExpectedPoint2 = Point3::new(0.29086178559218934, 4.1748550113475211, 2.2952614654595873);
    let (aCurvePoint, aSurfacePoint) = anExtrema.points(1).unwrap();
    assert!(aCurvePoint.is_equal(&anExpectedPoint1, 0.01));
    assert!(aSurfacePoint.is_equal(&anExpectedPoint1, 0.01));
    let (aCurvePoint, aSurfacePoint) = anExtrema.points(2).unwrap();
    assert!(aCurvePoint.is_equal(&anExpectedPoint2, 0.01));
    assert!(aSurfacePoint.is_equal(&anExpectedPoint2, 0.01));
    assert!(anExtrema.distance(3).unwrap() > 0.01);
    assert!(anExtrema.distance(4).unwrap() > 0.01);
}
