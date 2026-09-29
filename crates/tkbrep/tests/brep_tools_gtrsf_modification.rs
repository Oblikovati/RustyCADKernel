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

//! Direct translation of BRepTools_GTrsfModification_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871 (the declarations are split between two
//! included files, in their upstream order).
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{
    BRepToolsModification, GTrsfModification, Modifier, NewCurve, NewSurface,
};
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomBezierSurface, GeomCurve, GeomSurface};
use rustycad_tkmath::gp::{
    Axis2d, Direction2, Direction3, GeneralTransform3, Matrix3, Point2, Point3, Transform3,
    Vector3, Xyz,
};
use rustycad_tkmath::poly::{Polygon3D, PolygonOnTriangulation, Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;
use support::expect_double_eq;

fn makeTriangulation(theHasNormals: bool) -> Handle<Triangulation> {
    let aTriangulation = Triangulation::with_sizes(3, 1, false, theHasNormals).unwrap();
    aTriangulation
        .set_node(1, Point3::new(0.0, 0.0, 0.0))
        .unwrap();
    aTriangulation
        .set_node(2, Point3::new(1.0, 0.0, 0.0))
        .unwrap();
    aTriangulation
        .set_node(3, Point3::new(0.0, 1.0, 1.0))
        .unwrap();
    aTriangulation
        .set_triangle(1, Triangle::new(1, 2, 3))
        .unwrap();
    aTriangulation.set_deflection(0.25);
    if theHasNormals {
        let aNormal = Direction3::new(0.0, -1.0, 1.0).unwrap();
        for anIndex in 1..=3 {
            aTriangulation.set_normal(anIndex, aNormal).unwrap();
        }
    }
    Handle::new(aTriangulation)
}

/// `gp_Mat(a11, a12, a13, a21, a22, a23, a31, a32, a33)`.
fn gp_Mat(v: [f64; 9]) -> Matrix3 {
    Matrix3::new([[v[0], v[1], v[2]], [v[3], v[4], v[5]], [v[6], v[7], v[8]]])
}

fn makeGTrsf(theMatrix: Matrix3) -> GeneralTransform3 {
    let mut aGTrsf = GeneralTransform3::default();
    aGTrsf.set_vectorial_part(theMatrix);
    aGTrsf
}

fn expectPointNear(theActual: Point3, theExpected: Point3) {
    assert!(theActual.distance(&theExpected) < 1.0e-12);
}

/// `aGTrsf.Transforms(aPoint.ChangeCoord())`.
fn transforms(theGTrsf: &GeneralTransform3, thePoint: Point3) -> Point3 {
    let mut aCoord: Xyz = thePoint.xyz();
    theGTrsf.transforms(&mut aCoord);
    Point3::new(aCoord.x(), aCoord.y(), aCoord.z())
}

fn translation(x: f64, y: f64, z: f64) -> Transform3 {
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(x, y, z));
    aTrsf
}

fn bezierSurface(thePoles: [Point3; 4]) -> Handle<GeomBezierSurface> {
    let mut aPoles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aPoles.set_value(1, 1, thePoles[0]).unwrap();
    aPoles.set_value(2, 1, thePoles[1]).unwrap();
    aPoles.set_value(1, 2, thePoles[2]).unwrap();
    aPoles.set_value(2, 2, thePoles[3]).unwrap();
    Handle::new(GeomBezierSurface::new(&aPoles).unwrap())
}

fn bezierCurve(thePoles: &[Point3]) -> Handle<GeomBezierCurve> {
    let mut aPoles: Array1<'static, Point3> =
        Array1::with_bounds(1, thePoles.len() as i32).unwrap();
    for (anIndex, aPole) in thePoles.iter().enumerate() {
        aPoles.set_value(anIndex as i32 + 1, *aPole).unwrap();
    }
    Handle::new(GeomBezierCurve::new(&aPoles).unwrap())
}

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

include!("brep_tools_gtrsf_modification/suites_0.rs");
include!("brep_tools_gtrsf_modification/suites_1.rs");
