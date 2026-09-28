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

//! Direct translation of GProp_PGProps_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkgeombase::gprop::PGProps;
use rustycad_tkmath::gp::{Matrix3, Point3};

fn points(values: &[Point3]) -> Array1<'static, Point3> {
    let mut aPnts = Array1::with_bounds(1, values.len() as i32).unwrap();
    for (i, value) in values.iter().enumerate() {
        aPnts[i as i32 + 1] = *value;
    }
    aPnts
}

#[test]
fn GProp_PGPropsTest__EmptySet() {
    let aProps = PGProps::new();
    EXPECT_NEAR(aProps.mass(), 0.0, precision::confusion());
}

#[test]
fn GProp_PGPropsTest__SinglePoint() {
    let mut aProps = PGProps::new();
    aProps.add_point(Point3::new(1.0, 2.0, 3.0));

    EXPECT_NEAR(aProps.mass(), 1.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().x(), 1.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().y(), 2.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().z(), 3.0, precision::confusion());
}

#[test]
fn GProp_PGPropsTest__TwoPoints_Barycentre() {
    let mut aProps = PGProps::new();
    aProps.add_point(Point3::new(0.0, 0.0, 0.0));
    aProps.add_point(Point3::new(2.0, 4.0, 6.0));

    EXPECT_NEAR(aProps.mass(), 2.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().x(), 1.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().y(), 2.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().z(), 3.0, precision::confusion());
}

#[test]
fn GProp_PGPropsTest__WeightedPoints() {
    let mut aProps = PGProps::new();
    aProps
        .add_weighted_point(Point3::new(0.0, 0.0, 0.0), 1.0)
        .unwrap();
    aProps
        .add_weighted_point(Point3::new(4.0, 0.0, 0.0), 3.0)
        .unwrap();

    EXPECT_NEAR(aProps.mass(), 4.0, precision::confusion());
    // Weighted centroid: (0*1 + 4*3) / 4 = 3
    EXPECT_NEAR(aProps.centre_of_mass().x(), 3.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().y(), 0.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().z(), 0.0, precision::confusion());
}

#[test]
fn GProp_PGPropsTest__ArrayConstructor() {
    let aPnts = points(&[
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, -1.0, 0.0),
    ]);

    let aProps = PGProps::from_points(&aPnts);
    EXPECT_NEAR(aProps.mass(), 4.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().x(), 0.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().y(), 0.0, precision::confusion());
    EXPECT_NEAR(aProps.centre_of_mass().z(), 0.0, precision::confusion());
}

#[test]
fn GProp_PGPropsTest__StaticBarycentre() {
    let aPnts = points(&[
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(0.0, 6.0, 0.0),
    ]);

    let aG: Point3 = PGProps::barycentre(&aPnts).unwrap();
    EXPECT_NEAR(aG.x(), 1.0, precision::confusion());
    EXPECT_NEAR(aG.y(), 2.0, precision::confusion());
    EXPECT_NEAR(aG.z(), 0.0, precision::confusion());
}

#[test]
fn GProp_PGPropsTest__MatrixOfInertia_SymmetricPoints() {
    // Centroid is at the origin, so origin-frame and central-frame inertia
    // matrices coincide for these symmetric points.
    let aPnts = points(&[
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, -1.0, 0.0),
    ]);

    let aProps = PGProps::from_points(&aPnts);
    let anInertia: Matrix3 = aProps.matrix_of_inertia();

    EXPECT_NEAR(anInertia.value(1, 1).unwrap(), 2.0, precision::confusion()); // Ixx
    EXPECT_NEAR(anInertia.value(2, 2).unwrap(), 2.0, precision::confusion()); // Iyy
    EXPECT_NEAR(anInertia.value(3, 3).unwrap(), 4.0, precision::confusion()); // Izz
    EXPECT_NEAR(anInertia.value(1, 2).unwrap(), 0.0, precision::confusion()); // Ixy
    EXPECT_NEAR(anInertia.value(1, 3).unwrap(), 0.0, precision::confusion()); // Ixz
    EXPECT_NEAR(anInertia.value(2, 3).unwrap(), 0.0, precision::confusion()); // Iyz
}
