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

//! Direct translation of Plate_Plate_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkgeomalgo::plate::{PlatePinpointConstraint, PlatePlate};
use rustycad_tkmath::gp::{Xy, Xyz};

/// `EXPECT_DOUBLE_EQ`: equal within 4 units in the last place.
fn expect_double_eq(a: f64, b: f64) {
    let ulps = (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs();
    assert!(a == b || ulps <= 4, "{a} != {b}");
}

#[test]
fn Plate_Plate__Init_ClearsState() {
    let mut a_plate = PlatePlate::new();

    // Add a constraint and solve.
    a_plate.load_pinpoint(&PlatePinpointConstraint::new(
        Xy::new(0.0, 0.0),
        Xyz::new(0.0, 0.0, 1.0),
        0,
        0,
    ));
    a_plate.load_pinpoint(&PlatePinpointConstraint::new(
        Xy::new(1.0, 0.0),
        Xyz::new(0.0, 0.0, 0.0),
        0,
        0,
    ));
    a_plate.load_pinpoint(&PlatePinpointConstraint::new(
        Xy::new(0.0, 1.0),
        Xyz::new(0.0, 0.0, 0.0),
        0,
        0,
    ));
    a_plate.solve_ti(2, 1.0, &ProgressRange::default()).unwrap();

    // After Init(), constraints and solution are cleared.
    a_plate.init();
    // TODO: IsDone() returns true after Init() which is semantically wrong
    // (empty plate is not "done"). Setting OK=false in Init() caused regressions
    // in blend/filling DRAW tests. Needs investigation before fixing (finding #30).
    assert!(a_plate.is_done());
}

#[test]
fn Plate_Plate__Init_EvaluateReturnsZero() {
    let mut a_plate = PlatePlate::new();
    a_plate.init();

    let a_result: Xyz = a_plate.evaluate(Xy::new(0.5, 0.5)).unwrap();
    expect_double_eq(a_result.x(), 0.0);
    expect_double_eq(a_result.y(), 0.0);
    expect_double_eq(a_result.z(), 0.0);
}
