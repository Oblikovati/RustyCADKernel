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

//! OCC33048: port of the original order-125 memory regression.
#![allow(non_snake_case)]
use rustycad_tkmath::math::KronrodPointsAndWeights;
#[test]
fn math_ComputeKronrodPointsAndWeights_Test__OCC33048_ComputeWithOrder125() {
    // Rust panics and constructor failures fail this test, matching the native catch-all FAIL.
    let calculation: KronrodPointsAndWeights = KronrodPointsAndWeights::new(125).unwrap();
    assert!(
        calculation.is_done(),
        "Kronrod points and weights calculation should succeed"
    );
    let is_ok: bool = calculation.is_done();
    assert!(is_ok);
}
