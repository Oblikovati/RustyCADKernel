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

//! Direct translation of gce_MakeCirc2d_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gce::{ErrorType, MakeCirc2d, Root};
use rustycad_tkmath::gp::{self, Point2};

#[test]
fn gce_MakeCirc2dTest__ThreePoints_AllCoincidentWithinResolution_DoneZeroRadius() {
    // Intentional: points inside gp::Resolution() are treated as coincident
    // and should produce a valid zero-radius circle.
    let aDelta: f64 = 0.25 * gp::resolution();

    let aP1 = Point2::new(0.0, 0.0);
    let aP2 = Point2::new(aDelta, 0.0);
    let aP3 = Point2::new(0.0, aDelta);

    let aMaker = MakeCirc2d::through_points(aP1, aP2, aP3).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    EXPECT_NEAR(
        aMaker.value().unwrap().radius(),
        0.0,
        precision::confusion(),
    );
}

#[test]
fn gce_MakeCirc2dTest__ThreePoints_OneNearCoincidentPair_Done() {
    let aP1 = Point2::new(0.0, 0.0);
    let aP2 = Point2::new(1.0e-150, 0.0);
    let aP3 = Point2::new(0.0, 1.0);

    let aMaker = MakeCirc2d::through_points(aP1, aP2, aP3).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    assert!(aMaker.value().unwrap().radius() > 0.0);
}

#[test]
fn gce_MakeCirc2dTest__ThreePoints_SmallDistinct_Done() {
    let aP1 = Point2::new(0.0, 0.0);
    let aP2 = Point2::new(2.0e-150, 0.0);
    let aP3 = Point2::new(0.0, 1.0);

    let aMaker = MakeCirc2d::through_points(aP1, aP2, aP3).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
    assert!(aMaker.value().unwrap().radius() > 0.0);
}
