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

//! Direct translation of ModelingData/TKG3d/GTests/GeomAPI_ExtremaCurveCurve_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. The suite belongs to the TKG3d test
//! directory but tests GeomAPI (TKGeomAlgo), so it runs in this crate. An `ASSERT_*`
//! in a native helper returns from the helper only; the Rust helpers panic, which
//! differs only when a check fails.
#![allow(non_snake_case, clippy::excessive_precision)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCircle, GeomCurve, GeomGeometry, GeomLine, GeomSphericalSurface,
    GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkgeomalgo::geom_api::{ExtremaCurveCurve, ExtremaCurveSurface};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3, Transform3, Xyz};
use std::f64::consts::PI;

fn assert_near(left: f64, right: f64, tolerance: f64) {
    assert!((left - right).abs() <= tolerance, "{left} != {right}");
}

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

fn line(p: Point3, d: Direction3) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::from_point(p, d)).cast()
}

// `new Geom_TrimmedCurve(basis, u1, u2)` (the native default sense and periodic
// adjustment).
fn trimmed(basis: &Handle<dyn GeomCurve>, u1: f64, u2: f64) -> Handle<dyn GeomCurve> {
    Handle::new(GeomTrimmedCurve::new(basis, u1, u2, true, true).unwrap()).cast()
}

fn circle(center: Point3, normal: Direction3, radius: f64) -> Handle<GeomCircle> {
    Handle::new(GeomCircle::new(Axis2::new(center, normal).unwrap(), radius).unwrap())
}

fn checkOCC29426CircleSphere(theCircleCenterY: f64, theCircleCenterZ: f64, theSphereRadius: f64) {
    let frame = Axis2::with_x_direction(
        Point3::new(0.0, theCircleCenterY, theCircleCenterZ),
        dir(-3.67401855705945e-17, -0.943656647634893, 0.330926172090505),
        dir(-2.2723161366922e-16, 0.330926172090505, 0.943656647634893),
    )
    .unwrap();
    let aCircle: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(frame, 1.30000000000039).unwrap()).cast();
    let sphere_frame = Axis3::new(
        Point3::new(0.0, 2.425, 0.0),
        dir(0.0, 1.0, 0.0),
        dir(1.0, 0.0, 0.0),
    )
    .unwrap();
    let aSphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(sphere_frame, theSphereRadius).unwrap()).cast();

    let anExtrema = ExtremaCurveSurface::with_curve(&aCircle, &aSphere).unwrap();
    assert!(anExtrema.extrema().unwrap().is_done());

    if anExtrema.nb_extrema() > 0 {
        let mut aMinimumDistance = anExtrema.distance(1).unwrap();
        for anIndex in 2..=anExtrema.nb_extrema() {
            let aDistance = anExtrema.distance(anIndex).unwrap();
            aMinimumDistance = if aDistance < aMinimumDistance {
                aDistance
            } else {
                aMinimumDistance
            };
        }
        assert_near(aMinimumDistance, 0.0, 1.0e-10);
    }
}

fn makeLinearBSpline(thePoint1: Point3, thePoint2: Point3) -> Handle<dyn GeomCurve> {
    let mut aPoles: Array1<'static, Point3> = Array1::with_bounds(1, 2).unwrap();
    aPoles[1] = thePoint1;
    aPoles[2] = thePoint2;

    let mut aKnots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    aKnots[1] = 0.0;
    aKnots[2] = 1.0;

    let mut aMultiplicities: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    aMultiplicities[1] = 2;
    aMultiplicities[2] = 2;
    let curve = GeomBSplineCurve::new(&aPoles, &aKnots, &aMultiplicities, 1, false).unwrap();
    Handle::new(curve).cast()
}

fn checkOCC29712ConcentricCircles(
    theFirstCircleLastParameter: f64,
    theSecondCircleFirstParameter: f64,
    theSecondCircleLastParameter: f64,
) {
    let origin = Point3::new(0.0, 0.0, 0.0);
    let aCircle1 = circle(origin, dir(0.0, 0.0, 1.0), 100.0).cast::<dyn GeomCurve>();
    let aCircle2 = circle(origin, dir(0.0, 0.0, 1.0), 50.0).cast::<dyn GeomCurve>();
    let aTrimmed1 = trimmed(&aCircle1, 0.0, theFirstCircleLastParameter);
    let aTrimmed2 = trimmed(
        &aCircle2,
        theSecondCircleFirstParameter,
        theSecondCircleLastParameter,
    );

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aTrimmed1, &aTrimmed2).unwrap();
    assert!(anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_near(anExtrema12.lower_distance().unwrap(), 50.0, 1.0e-7);

    let anExtrema21 = ExtremaCurveCurve::with_curves(&aTrimmed2, &aTrimmed1).unwrap();
    assert!(anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_near(anExtrema21.lower_distance().unwrap(), 50.0, 1.0e-7);
}

fn checkParallelExtrema(
    theCurve1: &Handle<dyn GeomCurve>,
    theCurve2: &Handle<dyn GeomCurve>,
    theExpectedDistance: f64,
) {
    let anExtrema12 = ExtremaCurveCurve::with_curves(theCurve1, theCurve2).unwrap();
    assert!(anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_near(
        anExtrema12.lower_distance().unwrap(),
        theExpectedDistance,
        1.0e-8,
    );

    let anExtrema21 = ExtremaCurveCurve::with_curves(theCurve2, theCurve1).unwrap();
    assert!(anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_near(
        anExtrema21.lower_distance().unwrap(),
        theExpectedDistance,
        1.0e-8,
    );
}

fn checkOneFiniteExtremum(
    theCurve1: &Handle<dyn GeomCurve>,
    theCurve2: &Handle<dyn GeomCurve>,
    theExpectedDistance: f64,
) {
    let anExtrema12 = ExtremaCurveCurve::with_curves(theCurve1, theCurve2).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema12.nb_extrema(), 1);
    assert_near(
        anExtrema12.distance(1).unwrap(),
        theExpectedDistance,
        2.0e-8,
    );

    let anExtrema21 = ExtremaCurveCurve::with_curves(theCurve2, theCurve1).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema21.nb_extrema(), 1);
    assert_near(
        anExtrema21.distance(1).unwrap(),
        theExpectedDistance,
        2.0e-8,
    );
}

fn makeOCC29712TrimmedCircle(
    theRadius: f64,
    theFirstParameter: f64,
    theLastParameter: f64,
) -> Handle<dyn GeomCurve> {
    let aCircle = circle(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0), theRadius);
    trimmed(&aCircle.cast(), theFirstParameter, theLastParameter)
}

fn makeOCC29712MirroredTrimmedCircle(
    theRadius: f64,
    theFirstParameter: f64,
    theLastParameter: f64,
) -> Handle<dyn GeomCurve> {
    let aCircle = circle(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0), theRadius);
    let mut aMirror = Transform3::default();
    aMirror.set_mirror_axis(Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)));
    aCircle.transform(aMirror).unwrap();
    trimmed(&aCircle.cast(), theFirstParameter, theLastParameter)
}

fn checkFiniteConcentricExtremum(
    theSecondCircleFirstParameter: f64,
    theSecondCircleLastParameter: f64,
) {
    let aCurve1 = makeOCC29712TrimmedCircle(100.0, 0.0, 3.0);
    let aCurve2 = makeOCC29712TrimmedCircle(
        50.0,
        theSecondCircleFirstParameter,
        theSecondCircleLastParameter,
    );

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aCurve1, &aCurve2).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema12.nb_extrema(), 1);
    assert_near(anExtrema12.distance(1).unwrap(), 50.0, 1.0e-7);

    let anExtrema21 = ExtremaCurveCurve::with_curves(&aCurve2, &aCurve1).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert_eq!(anExtrema21.nb_extrema(), 1);
    assert_near(anExtrema21.distance(1).unwrap(), 50.0, 1.0e-7);
}

fn checkFiniteConcentricExtremumExists(
    theSecondCircleFirstParameter: f64,
    theSecondCircleLastParameter: f64,
) {
    let aCurve1 = makeOCC29712TrimmedCircle(100.0, 0.0, 3.0);
    let aCurve2 = makeOCC29712TrimmedCircle(
        50.0,
        theSecondCircleFirstParameter,
        theSecondCircleLastParameter,
    );

    let anExtrema12 = ExtremaCurveCurve::with_curves(&aCurve1, &aCurve2).unwrap();
    assert!(!anExtrema12.extrema().unwrap().is_parallel().unwrap());
    assert!(anExtrema12.nb_extrema() > 0);

    let anExtrema21 = ExtremaCurveCurve::with_curves(&aCurve2, &aCurve1).unwrap();
    assert!(!anExtrema21.extrema().unwrap().is_parallel().unwrap());
    assert!(anExtrema21.nb_extrema() > 0);
}

include!("tkg3d_geomapi_extrema_curve_curve/occ862_data.rs");
include!("tkg3d_geomapi_extrema_curve_curve/suites_1.rs");
include!("tkg3d_geomapi_extrema_curve_curve/suites_2.rs");
include!("tkg3d_geomapi_extrema_curve_curve/suites_3.rs");
