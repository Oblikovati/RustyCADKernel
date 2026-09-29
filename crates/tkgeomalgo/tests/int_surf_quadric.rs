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

//! Direct translation of IntSurf_Quadric_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::precision;
use rustycad_tkgeomalgo::int_surf::IntSurfQuadric;
use rustycad_tkmath::gp::{Axis3, Cone3, Direction3, Point3, Vector3};

#[test]
fn IntSurf_Quadric_Test__ConeApexGradientRemainsFinite() {
    let a_cone = Cone3::new(
        Axis3::new(
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
            Direction3::new(1.0, 0.0, 0.0).unwrap(),
        )
        .unwrap(),
        0.5,
        0.0,
    )
    .unwrap();
    let a_quadric = IntSurfQuadric::from_cone(&a_cone);
    let an_apex: Point3 = a_cone.apex();

    // EXPECT_NO_THROW: the Rust members cannot raise here, and must not panic.
    {
        let a_gradient: Vector3 = a_quadric.gradient(an_apex);
        assert!(a_gradient.square_magnitude() <= precision::square_confusion());
    }

    let mut a_dist: f64 = 1.0;
    let mut a_grad = Vector3::new(1.0, 0.0, 0.0);
    if let Some((dist, grad)) = a_quadric.val_and_grad(an_apex) {
        (a_dist, a_grad) = (dist, grad);
    }
    assert!((a_dist - 0.0).abs() <= precision::confusion());
    assert!(a_grad.square_magnitude() <= precision::square_confusion());
}
