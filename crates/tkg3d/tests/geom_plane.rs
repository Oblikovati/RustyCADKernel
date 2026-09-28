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

//! Direct translation of Geom_Plane_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomElementarySurface, GeomGeometry, GeomLine, GeomPlane, GeomSurface,
};
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Transform3, Vector3};

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

// `new Geom_Plane(P, gp_Dir(0, 0, 1))`.
fn z_plane(x: f64, y: f64, z: f64) -> Handle<GeomPlane> {
    Handle::new(GeomPlane::from_point(Point3::new(x, y, z), dir(0.0, 0.0, 1.0)).unwrap())
}

#[test]
fn Geom_PlaneTest__ConstructFromAx3() {
    let anAx3 = Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap();
    let aPlane = Handle::new(GeomPlane::new(anAx3));
    assert!(!aPlane.is_null());
    assert!(
        aPlane
            .location()
            .is_equal(&Point3::new(0.0, 0.0, 0.0), precision::confusion())
    );
}

#[test]
fn Geom_PlaneTest__ConstructFromPointAndDir() {
    let aPlane = z_plane(1.0, 2.0, 3.0);
    assert!(!aPlane.is_null());
    assert!(
        aPlane
            .location()
            .is_equal(&Point3::new(1.0, 2.0, 3.0), precision::confusion())
    );
}

#[test]
fn Geom_PlaneTest__D0Evaluation() {
    let aPlane = z_plane(0.0, 0.0, 0.0);

    // XY plane at origin: D0(U,V) = (U,V,0) for default axis
    let aPnt: Point3 = aPlane.d0(3.0, 4.0).unwrap();
    EXPECT_NEAR(aPnt.x(), 3.0, precision::confusion());
    EXPECT_NEAR(aPnt.y(), 4.0, precision::confusion());
    EXPECT_NEAR(aPnt.z(), 0.0, precision::confusion());
}

#[test]
fn Geom_PlaneTest__Coefficients() {
    let aPlane = z_plane(0.0, 0.0, 5.0);
    let [anA, aB, aC, aD] = aPlane.coefficients().unwrap();
    EXPECT_NEAR(anA, 0.0, precision::confusion());
    EXPECT_NEAR(aB, 0.0, precision::confusion());
    EXPECT_NEAR(aC, 1.0, precision::confusion());
    EXPECT_NEAR(aD, -5.0, precision::confusion());
}

#[test]
fn Geom_PlaneTest__UIso() {
    let aPlane = z_plane(0.0, 0.0, 0.0);
    let aUIso: Handle<dyn GeomCurve> = aPlane.u_iso(2.0).unwrap();
    assert!(!aUIso.is_null());

    // UIso at U=2 on XY plane should be a line at x=2, varying in Y
    let aLine: Handle<GeomLine> = aUIso.cast();
    assert!(!aLine.is_null());

    let aPnt: Point3 = aLine.d0(0.0).unwrap();
    EXPECT_NEAR(aPnt.x(), 2.0, precision::confusion());
}

#[test]
fn Geom_PlaneTest__VIso() {
    let aPlane = z_plane(0.0, 0.0, 0.0);
    let aVIso: Handle<dyn GeomCurve> = aPlane.v_iso(3.0).unwrap();
    assert!(!aVIso.is_null());

    // VIso at V=3 on XY plane should be a line at y=3, varying in X
    let aLine: Handle<GeomLine> = aVIso.cast();
    assert!(!aLine.is_null());

    let aPnt: Point3 = aLine.d0(0.0).unwrap();
    EXPECT_NEAR(aPnt.y(), 3.0, precision::confusion());
}

#[test]
fn Geom_PlaneTest__Transform() {
    let aPlane = z_plane(0.0, 0.0, 0.0);
    let mut aTrsf = Transform3::default();
    aTrsf.set_translation(Vector3::new(0.0, 0.0, 10.0));
    aPlane.transform(aTrsf).unwrap();
    EXPECT_NEAR(aPlane.location().z(), 10.0, precision::confusion());
}

#[test]
fn Geom_PlaneTest__Copy() {
    let aPlane = z_plane(1.0, 2.0, 3.0);
    let aCopy: Handle<dyn GeomGeometry> = aPlane.copy().unwrap();
    assert!(!aCopy.is_null());
    let aCopyPlane: Handle<GeomPlane> = aCopy.cast();
    assert!(!aCopyPlane.is_null());
    assert!(
        aCopyPlane
            .location()
            .is_equal(&Point3::new(1.0, 2.0, 3.0), precision::confusion())
    );
}
