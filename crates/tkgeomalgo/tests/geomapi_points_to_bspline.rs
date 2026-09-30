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

//! Direct translation of GeomAPI_PointsToBSpline_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkgeomalgo::geom_api::PointsToBSpline;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

#[test]
fn GeomAPI_PointsToBSpline_Test__DegenerateExplicitParametersResetDoneState() {
    let mut aPoints: Array1<'static, Point3> = Array1::with_bounds(1, 3).unwrap();
    aPoints[1] = Point3::new(0.0, 0.0, 0.0);
    aPoints[2] = Point3::new(1.0, 0.5, 0.0);
    aPoints[3] = Point3::new(2.0, 0.0, 0.0);

    let mut aParameters: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    aParameters[1] = 2.0;
    aParameters[2] = 2.0;
    aParameters[3] = 2.0;

    let mut anApprox =
        PointsToBSpline::with_points(&aPoints, 3, 5, Shape::C2, precision::confusion()).unwrap();
    assert!(anApprox.is_done());

    // EXPECT_NO_THROW
    assert!(
        anApprox
            .init_parameters(
                &aPoints,
                &aParameters,
                3,
                5,
                Shape::C2,
                precision::confusion()
            )
            .is_ok()
    );
    assert!(!anApprox.is_done());
}
