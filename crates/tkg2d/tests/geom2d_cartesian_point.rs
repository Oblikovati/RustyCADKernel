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

//! Direct translation of Geom2d_CartesianPoint_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dGeometry, Geom2dPoint};
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use std::f64::consts::PI;

#[test]
fn Geom2d_CartesianPointTest__ConstructFromCoords() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(3.0, 4.0));
    EXPECT_NEAR(aPnt.x(), 3.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 4.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__ConstructFromPnt2d() {
    let aPnt = Handle::new(Geom2dCartesianPoint::from_pnt2d(Point2::new(5.0, 6.0)));
    EXPECT_NEAR(aPnt.x(), 5.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 6.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__SetCoord() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(0.0, 0.0));
    aPnt.set_coord(7.0, 8.0);
    EXPECT_NEAR(aPnt.x(), 7.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 8.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__SetPnt2d() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(0.0, 0.0));
    aPnt.set_pnt2d(Point2::new(1.0, 2.0));
    EXPECT_NEAR(aPnt.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 2.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__SetX_SetY() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(0.0, 0.0));
    aPnt.set_x(10.0);
    aPnt.set_y(20.0);
    EXPECT_NEAR(aPnt.x(), 10.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 20.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__Coord() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(3.0, 4.0));
    let (aX, aY): (f64, f64) = aPnt.coord();
    EXPECT_NEAR(aX, 3.0, precision::confusion());
    EXPECT_NEAR(aY, 4.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__Pnt2d() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(3.0, 4.0));
    let aGpPnt: Point2 = aPnt.pnt2d();
    EXPECT_NEAR(aGpPnt.x(), 3.0, precision::confusion());
    EXPECT_NEAR(aGpPnt.y(), 4.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__Distance() {
    let aP1 = Handle::new(Geom2dCartesianPoint::new(0.0, 0.0));
    let aP2 = Handle::new(Geom2dCartesianPoint::new(3.0, 4.0));
    EXPECT_NEAR(aP1.distance(&*aP2), 5.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__SquareDistance() {
    let aP1 = Handle::new(Geom2dCartesianPoint::new(0.0, 0.0));
    let aP2 = Handle::new(Geom2dCartesianPoint::new(3.0, 4.0));
    EXPECT_NEAR(aP1.square_distance(&*aP2), 25.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__Distance_SamePoint() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(5.0, 5.0));
    EXPECT_NEAR(aPnt.distance(&*aPnt), 0.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__Transform_Translation() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(1.0, 2.0));

    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(3.0, 4.0));
    aPnt.transform(aTrsf).unwrap();

    EXPECT_NEAR(aPnt.x(), 4.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 6.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__Transform_Rotation() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(1.0, 0.0));

    let mut aTrsf = Transform2::default();
    aTrsf.set_rotation(Point2::new(0.0, 0.0), PI / 2.0);
    aPnt.transform(aTrsf).unwrap();

    EXPECT_NEAR(aPnt.x(), 0.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_CartesianPointTest__Copy() {
    let aPnt = Handle::new(Geom2dCartesianPoint::new(3.0, 4.0));
    let aCopyGeom: Handle<dyn Geom2dGeometry> = aPnt.copy().unwrap();
    let aCopy: Handle<Geom2dCartesianPoint> = aCopyGeom.cast::<Geom2dCartesianPoint>();

    assert!(!aCopy.is_null());
    EXPECT_NEAR(aCopy.x(), 3.0, precision::confusion());
    EXPECT_NEAR(aCopy.y(), 4.0, precision::confusion());

    aCopy.set_coord(0.0, 0.0);
    EXPECT_NEAR(aPnt.x(), 3.0, precision::confusion());
}
