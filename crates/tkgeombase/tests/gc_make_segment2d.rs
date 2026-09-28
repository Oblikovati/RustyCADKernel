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

//! Direct translation of GC_MakeSegment2d_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkgeombase::gc::{ErrorType, MakeSegment2d, Root};
use rustycad_tkmath::gp::{self, Direction2, Point2};

fn x_dir() -> Direction2 {
    Direction2::new(1.0, 0.0).unwrap()
}

#[test]
fn GC_MakeSegment2dTest__FromPoints_BelowResolution_ConfusedPoints() {
    let aP1 = Point2::new(0.0, 0.0);
    let aP2 = Point2::new(0.5 * gp::resolution(), 0.0);

    let aMaker = MakeSegment2d::from_points(aP1, aP2).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}

#[test]
fn GC_MakeSegment2dTest__FromPoints_AboveResolution_Done() {
    let aP1 = Point2::new(0.0, 0.0);
    let aP2 = Point2::new(1.0e-150, 0.0);

    let aMaker = MakeSegment2d::from_points(aP1, aP2).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
}

#[test]
fn GC_MakeSegment2dTest__FromPoints_AtResolution_ConfusedPoints() {
    let aP1 = Point2::new(0.0, 0.0);
    let aP2 = Point2::new(gp::resolution(), 0.0);

    let aMaker = MakeSegment2d::from_points(aP1, aP2).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}

#[test]
fn GC_MakeSegment2dTest__FromPointDir_BelowResolutionParameter_ConfusedPoints() {
    let aP1 = Point2::new(0.0, 0.0);
    let aDir = x_dir();
    let aP2 = Point2::new(0.5 * gp::resolution(), 3.0);

    let aMaker = MakeSegment2d::from_direction(aP1, aDir, aP2).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}

#[test]
fn GC_MakeSegment2dTest__FromPointDir_AboveResolutionParameter_Done() {
    let aP1 = Point2::new(0.0, 0.0);
    let aDir = x_dir();
    let aP2 = Point2::new(2.0 * gp::resolution(), 3.0);

    let aMaker = MakeSegment2d::from_direction(aP1, aDir, aP2).unwrap();
    assert!(aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::Done);
}

#[test]
fn GC_MakeSegment2dTest__FromPointDir_AtResolutionParameter_ConfusedPoints() {
    let aP1 = Point2::new(0.0, 0.0);
    let aDir = x_dir();
    let aP2 = Point2::new(gp::resolution(), 5.0);

    let aMaker = MakeSegment2d::from_direction(aP1, aDir, aP2).unwrap();
    assert!(!aMaker.is_done());
    assert_eq!(aMaker.status(), ErrorType::ConfusedPoints);
}
