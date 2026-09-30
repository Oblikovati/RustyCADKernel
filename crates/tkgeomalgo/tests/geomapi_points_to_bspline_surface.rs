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

//! Direct translation of GeomAPI_PointsToBSplineSurface_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::precision;
use rustycad_tkgeomalgo::geom_api::PointsToBSplineSurface;
use rustycad_tkmath::geom_abs::Shape;

#[test]
fn GeomAPI_PointsToBSplineSurface_Test__FailedDegenerateRebuildResetsDoneState() {
    let mut aValidZPoints: Array2<'static, f64> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    aValidZPoints[(1, 1)] = 0.0;
    aValidZPoints[(1, 2)] = 1.0;
    aValidZPoints[(2, 1)] = 1.0;
    aValidZPoints[(2, 2)] = 0.0;

    let mut anApprox = PointsToBSplineSurface::new();
    // EXPECT_NO_THROW
    assert!(
        anApprox
            .init_heights(
                &aValidZPoints,
                (0.0, 1.0, 0.0, 1.0),
                (3, 5, Shape::C2, precision::confusion())
            )
            .is_ok()
    );
    assert!(anApprox.is_done());

    let mut aZPoints: Array2<'static, f64> = Array2::with_bounds(1, 3, 1, 1).unwrap();
    aZPoints[(1, 1)] = 0.0;
    aZPoints[(2, 1)] = 1.0;
    aZPoints[(3, 1)] = 0.0;

    // EXPECT_THROW(..., StdFail_NotDone)
    let thrown = anApprox
        .init_heights(
            &aZPoints,
            (0.0, 1.0, 0.0, 0.0),
            (3, 5, Shape::C2, precision::confusion()),
        )
        .unwrap_err();
    assert_eq!(thrown.exception_type(), "StdFail_NotDone");
    assert!(!anApprox.is_done());
}
