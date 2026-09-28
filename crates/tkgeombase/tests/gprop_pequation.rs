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

//! Direct translation of GProp_PEquation_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gprop::PEquation;
use rustycad_tkmath::gp::{Line3, Plane3, Point3};

fn points(values: &[Point3]) -> Array1<'static, Point3> {
    let mut aPnts = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, value) in values.iter().enumerate() {
        aPnts[i as i32 + 1] = *value;
    }
    aPnts
}

#[test]
fn GProp_PEquationTest__CoincidentPoints() {
    let aPnts = points(&[
        Point3::new(1.0, 2.0, 3.0),
        Point3::new(1.0, 2.0, 3.0),
        Point3::new(1.0, 2.0, 3.0),
    ]);

    let anEq = PEquation::new(&aPnts, 1e-6);
    assert!(anEq.is_point());

    let aP: Point3 = anEq.point().unwrap();
    EXPECT_NEAR(aP.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aP.y(), 2.0, precision::confusion());
    EXPECT_NEAR(aP.z(), 3.0, precision::confusion());
}

#[test]
fn GProp_PEquationTest__CollinearPoints() {
    let aPnts = points(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
    ]);

    let anEq = PEquation::new(&aPnts, 1e-6);
    assert!(anEq.is_linear());

    let aLin: Line3 = anEq.line().unwrap();
    EXPECT_NEAR(aLin.direction().x().abs(), 1.0, precision::confusion());
}

#[test]
fn GProp_PEquationTest__CoplanarPoints() {
    let aPnts = points(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ]);

    let anEq = PEquation::new(&aPnts, 1e-6);
    assert!(anEq.is_planar());

    let aPln: Plane3 = anEq.plane().unwrap();
    EXPECT_NEAR(
        aPln.axis().direction().z().abs(),
        1.0,
        precision::confusion(),
    );
}

#[test]
fn GProp_PEquationTest__SpacePoints() {
    let aPnts = points(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
    ]);

    let anEq = PEquation::new(&aPnts, 1e-6);
    assert!(anEq.is_space());

    let (_aP, aV1, aV2, aV3) = anEq.box_().unwrap();
    assert!(aV1.magnitude() > precision::confusion());
    assert!(aV2.magnitude() > precision::confusion());
    assert!(aV3.magnitude() > precision::confusion());
}
