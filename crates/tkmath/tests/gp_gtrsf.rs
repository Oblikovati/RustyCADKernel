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

//! OCC13963: plane affinity leaves identity intact and scales the normal component.
#![allow(non_snake_case)]
use rustycad_tkmath::gp::{Axis2, Direction3, GeneralTransform3, Point3, Xyz};
#[test]
fn gp_GTrsfTest__OCC13963_SetAffinity_Ratio1_IsIdentity() {
    let plane = Axis2::new(Point3::default(), Direction3::new(1.0, -1.0, 0.0).unwrap()).unwrap();
    let mut transform = GeneralTransform3::default();
    transform.set_affinity_plane(plane, 1.0);
    let origin = Xyz::new(1.0, 0.0, 0.0);
    let mut result = origin;
    transform.transforms(&mut result);
    assert!((result.x() - 1.0).abs() <= 1e-7);
    assert!(result.y().abs() <= 1e-7);
    assert!(result.z().abs() <= 1e-7);
}
#[test]
fn gp_GTrsfTest__OCC13963_SetAffinity_Ratio2_ScalesAlongNormal() {
    let plane = Axis2::new(Point3::default(), Direction3::new(1.0, -1.0, 0.0).unwrap()).unwrap();
    let mut transform = GeneralTransform3::default();
    transform.set_affinity_plane(plane, 2.0);
    let origin = Xyz::new(1.0, 0.0, 0.0);
    let mut result = origin;
    transform.transforms(&mut result);
    assert!((result.x() - 1.5).abs() <= 1e-7);
    assert!((result.y() + 0.5).abs() <= 1e-7);
    assert!(result.z().abs() <= 1e-7);
}
