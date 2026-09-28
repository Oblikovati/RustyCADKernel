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

//! Direct translation of Geom2d_AxisPlacement_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dAxisPlacement, Geom2dGeometry};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2, Vector2};
use std::f64::consts::PI;

fn dir(x: f64, y: f64) -> Direction2 {
    Direction2::new(x, y).unwrap()
}
fn placement(x: f64, y: f64, dx: f64, dy: f64) -> Handle<Geom2dAxisPlacement> {
    Handle::new(Geom2dAxisPlacement::from_location(
        Point2::new(x, y),
        dir(dx, dy),
    ))
}

#[test]
fn Geom2d_AxisPlacementTest__ConstructFromAx2d() {
    let anAxis = Axis2d::new(Point2::new(1.0, 2.0), dir(1.0, 0.0));
    let anAP = Handle::new(Geom2dAxisPlacement::new(anAxis));

    EXPECT_NEAR(anAP.location().x(), 1.0, precision::confusion());
    EXPECT_NEAR(anAP.location().y(), 2.0, precision::confusion());
    EXPECT_NEAR(anAP.direction().x(), 1.0, precision::confusion());
    EXPECT_NEAR(anAP.direction().y(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__ConstructFromPointAndDir() {
    let anAP = placement(3.0, 4.0, 0.0, 1.0);

    EXPECT_NEAR(anAP.location().x(), 3.0, precision::confusion());
    EXPECT_NEAR(anAP.direction().y(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__SetLocation() {
    let anAP = placement(0.0, 0.0, 1.0, 0.0);
    anAP.set_location(Point2::new(5.0, 6.0));

    EXPECT_NEAR(anAP.location().x(), 5.0, precision::confusion());
    EXPECT_NEAR(anAP.location().y(), 6.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__SetDirection() {
    let anAP = placement(0.0, 0.0, 1.0, 0.0);
    anAP.set_direction(dir(0.0, 1.0));

    EXPECT_NEAR(anAP.direction().x(), 0.0, precision::confusion());
    EXPECT_NEAR(anAP.direction().y(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__SetAxis() {
    let anAP = placement(0.0, 0.0, 1.0, 0.0);

    let aNewAxis = Axis2d::new(Point2::new(10.0, 20.0), dir(0.0, 1.0));
    anAP.set_axis(aNewAxis);

    EXPECT_NEAR(anAP.location().x(), 10.0, precision::confusion());
    EXPECT_NEAR(anAP.direction().y(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__Ax2d() {
    let anAP = placement(1.0, 2.0, 1.0, 0.0);

    let anAxis: Axis2d = anAP.ax2d();
    EXPECT_NEAR(anAxis.location().x(), 1.0, precision::confusion());
    EXPECT_NEAR(anAxis.direction().x(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__Angle_Perpendicular() {
    let anAP1 = placement(0.0, 0.0, 1.0, 0.0);
    let anAP2 = placement(0.0, 0.0, 0.0, 1.0);

    EXPECT_NEAR(anAP1.angle(&anAP2), PI / 2.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__Angle_Parallel() {
    let anAP1 = placement(0.0, 0.0, 1.0, 0.0);
    let anAP2 = placement(5.0, 5.0, 1.0, 0.0);

    EXPECT_NEAR(anAP1.angle(&anAP2), 0.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__Angle_Opposite() {
    let anAP1 = placement(0.0, 0.0, 1.0, 0.0);
    let anAP2 = placement(0.0, 0.0, -1.0, 0.0);

    EXPECT_NEAR(anAP1.angle(&anAP2).abs(), PI, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__Reverse() {
    let anAP = placement(0.0, 0.0, 1.0, 0.0);
    anAP.reverse();

    EXPECT_NEAR(anAP.direction().x(), -1.0, precision::confusion());
    EXPECT_NEAR(anAP.direction().y(), 0.0, precision::confusion());
    // Location unchanged
    EXPECT_NEAR(anAP.location().x(), 0.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__Reversed() {
    let anAP = placement(0.0, 0.0, 1.0, 0.0);
    let aReversed: Handle<Geom2dAxisPlacement> = anAP.reversed();

    EXPECT_NEAR(aReversed.direction().x(), -1.0, precision::confusion());
    // Original unchanged
    EXPECT_NEAR(anAP.direction().x(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__Transform_Translation() {
    let anAP = placement(1.0, 2.0, 1.0, 0.0);

    let mut aTrsf = Transform2::default();
    aTrsf.set_translation(Vector2::new(10.0, 20.0));
    anAP.transform(aTrsf).unwrap();

    EXPECT_NEAR(anAP.location().x(), 11.0, precision::confusion());
    EXPECT_NEAR(anAP.location().y(), 22.0, precision::confusion());
    // Direction unchanged
    EXPECT_NEAR(anAP.direction().x(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__Transform_Rotation() {
    let anAP = placement(1.0, 0.0, 1.0, 0.0);

    let mut aTrsf = Transform2::default();
    aTrsf.set_rotation(Point2::new(0.0, 0.0), PI / 2.0);
    anAP.transform(aTrsf).unwrap();

    EXPECT_NEAR(anAP.location().x(), 0.0, precision::confusion());
    EXPECT_NEAR(anAP.location().y(), 1.0, precision::confusion());
    EXPECT_NEAR(anAP.direction().x(), 0.0, precision::confusion());
    EXPECT_NEAR(anAP.direction().y(), 1.0, precision::confusion());
}

#[test]
fn Geom2d_AxisPlacementTest__Copy() {
    let anAP = placement(1.0, 2.0, 0.0, 1.0);
    let aCopyGeom: Handle<dyn Geom2dGeometry> = anAP.copy().unwrap();
    let aCopy: Handle<Geom2dAxisPlacement> = aCopyGeom.cast::<Geom2dAxisPlacement>();

    assert!(!aCopy.is_null());
    EXPECT_NEAR(aCopy.location().x(), 1.0, precision::confusion());
    EXPECT_NEAR(aCopy.direction().y(), 1.0, precision::confusion());

    // Verify independence
    aCopy.set_location(Point2::new(99.0, 99.0));
    EXPECT_NEAR(anAP.location().x(), 1.0, precision::confusion());
}
