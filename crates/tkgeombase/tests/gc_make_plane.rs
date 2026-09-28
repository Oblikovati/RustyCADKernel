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

//! Direct translation of GC_MakePlane_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gc::{ErrorType, MakePlane, Root};
use rustycad_tkmath::gp::{Axis1, Direction3, Plane3, Point3};

fn base_plane() -> Plane3 {
    Plane3::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap()
}

#[test]
fn GC_MakePlaneTest__FromPlaneAndDist_Done() {
    let aBase = base_plane();

    let aMaker = MakePlane::from_plane_distance(aBase, 5.0).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    EXPECT_NEAR(
        aMaker.value().unwrap().pln().location().z(),
        5.0,
        precision::confusion(),
    );
}

#[test]
fn GC_MakePlaneTest__FromPlaneAndPoint_Done() {
    let aBase = base_plane();
    let aPoint = Point3::new(2.0, -3.0, 7.0);

    let aMaker = MakePlane::from_plane_point(aBase, aPoint).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    let aLocation: Point3 = aMaker.value().unwrap().pln().location();
    assert!(aLocation.is_equal(&aPoint, precision::confusion()));
}

#[test]
fn GC_MakePlaneTest__FromAxis_Done() {
    let aAxis = Axis1::new(
        Point3::new(1.0, 2.0, 3.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
    );

    let aMaker = MakePlane::from_axis(aAxis).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    let aNormal: Direction3 = aMaker.value().unwrap().pln().axis().direction();
    assert!(aNormal.is_parallel(aAxis.direction(), precision::angular()));
}
