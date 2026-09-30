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

//! Direct translation of GeomAPI_IntCS_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomCircle, GeomCurve, GeomParabola, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkgeomalgo::geom_api::IntCS;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3};

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

fn makeOCC26979Curve() -> Handle<dyn GeomCurve> {
    let frame = Axis2::new(Point3::new(-5.0, -1.0, 0.25), dir(0.0, 0.0, 1.0)).unwrap();
    let aParabola: Handle<dyn GeomCurve> =
        Handle::new(GeomParabola::new(frame, 0.25).unwrap()).cast();
    Handle::new(GeomTrimmedCurve::new(&aParabola, 0.0, 1000.0, true, true).unwrap()).cast()
}

fn expectOCC26979Intersection(theSurface: &Handle<dyn GeomSurface>) {
    let anIntersection = IntCS::with_curve(&makeOCC26979Curve(), theSurface).unwrap();
    assert!(anIntersection.is_done());
    assert!(anIntersection.nb_points().unwrap() + anIntersection.nb_segments().unwrap() > 0);
}

fn makeOCC26979BezierSurface() -> Handle<dyn GeomSurface> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    aPoles[(1, 1)] = Point3::new(-4.0, 0.0, -1.0);
    aPoles[(2, 1)] = Point3::new(-4.0, 1.0, 0.0);
    aPoles[(3, 1)] = Point3::new(-4.0, 0.0, 1.0);
    aPoles[(1, 2)] = Point3::new(-3.0, 1.0, -1.0);
    aPoles[(2, 2)] = Point3::new(-3.0, 2.0, 0.0);
    aPoles[(3, 2)] = Point3::new(-3.0, 1.0, 1.0);
    aPoles[(1, 3)] = Point3::new(-2.0, 0.0, -1.0);
    aPoles[(2, 3)] = Point3::new(-2.0, 1.0, 0.0);
    aPoles[(3, 3)] = Point3::new(-2.0, 0.0, 1.0);
    Handle::new(GeomBezierSurface::new(&aPoles).unwrap()).cast()
}

fn makeOCC26979BSplineSurface() -> Handle<dyn GeomSurface> {
    let mut aUKnots: Array1<'static, f64> = Array1::with_bounds(1, 5).unwrap();
    let mut aVKnots: Array1<'static, f64> = Array1::with_bounds(1, 5).unwrap();
    let mut aUMultiplicities: Array1<'static, i32> = Array1::with_bounds(1, 5).unwrap();
    let mut aVMultiplicities: Array1<'static, i32> = Array1::with_bounds(1, 5).unwrap();
    for anIndex in 1..=5 {
        aUKnots[anIndex] = f64::from(anIndex - 1);
        aVKnots[anIndex] = f64::from(anIndex - 1);
        aUMultiplicities[anIndex] = 1;
        aVMultiplicities[anIndex] = 1;
    }

    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    aPoles[(1, 1)] = Point3::new(-3.0, 0.5, -1.0);
    aPoles[(2, 1)] = Point3::new(-2.0, 0.5, 0.0);
    aPoles[(3, 1)] = Point3::new(-3.0, 0.5, 1.0);
    aPoles[(1, 2)] = Point3::new(-2.0, 1.5, -1.0);
    aPoles[(2, 2)] = Point3::new(-1.0, 1.5, 0.0);
    aPoles[(3, 2)] = Point3::new(-2.0, 1.5, 1.0);
    aPoles[(1, 3)] = Point3::new(-3.0, 2.5, -1.0);
    aPoles[(2, 3)] = Point3::new(-2.0, 2.5, 0.0);
    aPoles[(3, 3)] = Point3::new(-3.0, 2.5, 1.0);

    let mut aWeights: Array2<'static, f64> = Array2::with_bounds(1, 3, 1, 3).unwrap();
    for aU in 1..=3 {
        for aV in 1..=3 {
            aWeights[(aU, aV)] = 1.0;
        }
    }
    let surface = GeomBSplineSurface::with_weights(
        &aPoles,
        &aWeights,
        (&aUKnots, &aUMultiplicities),
        (&aVKnots, &aVMultiplicities),
        (1, 1),
        (false, false),
    );
    Handle::new(surface.unwrap()).cast()
}

fn makeOCC26979RevolutionSurface() -> Handle<dyn GeomSurface> {
    let frame = Axis2::with_x_direction(
        Point3::new(0.0, 0.0, 0.0),
        dir(0.0, -1.0, 0.0),
        dir(1.0, 0.0, 0.0),
    )
    .unwrap();
    let aCircle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(frame, 3.0).unwrap()).cast();
    let half_pi: f64 = std::f64::consts::PI / 2.0;
    let aTrimmedCircle: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(&aCircle, -half_pi, half_pi, true, true).unwrap()).cast();
    let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0));
    Handle::new(GeomSurfaceOfRevolution::new(&aTrimmedCircle, axis).unwrap()).cast()
}

// Migrated from tests/lowalgos/intcs/bug26979.
#[test]
fn GeomAPI_IntCSTest__OCC26979_ParabolaBezierSurface() {
    expectOCC26979Intersection(&makeOCC26979BezierSurface());
}

// Migrated from tests/lowalgos/intcs/bug26979.
#[test]
fn GeomAPI_IntCSTest__OCC26979_ParabolaBSplineSurface() {
    expectOCC26979Intersection(&makeOCC26979BSplineSurface());
}

// Migrated from tests/lowalgos/intcs/bug26979.
#[test]
fn GeomAPI_IntCSTest__OCC26979_ParabolaTorus() {
    let frame = Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap();
    let torus: Handle<dyn GeomSurface> =
        Handle::new(GeomToroidalSurface::new(frame, 3.0, 1.0).unwrap()).cast();
    expectOCC26979Intersection(&torus);
}

// Migrated from tests/lowalgos/intcs/bug26979.
#[test]
fn GeomAPI_IntCSTest__OCC26979_ParabolaLinearExtrusion() {
    let frame = Axis2::new(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap();
    let aCircle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(frame, 4.0).unwrap()).cast();
    let extrusion: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfLinearExtrusion::new(&aCircle, dir(0.0, 0.0, 1.0)).unwrap())
            .cast();
    expectOCC26979Intersection(&extrusion);
}

// Migrated from tests/lowalgos/intcs/bug26979.
#[test]
fn GeomAPI_IntCSTest__OCC26979_ParabolaRevolution() {
    expectOCC26979Intersection(&makeOCC26979RevolutionSurface());
}
