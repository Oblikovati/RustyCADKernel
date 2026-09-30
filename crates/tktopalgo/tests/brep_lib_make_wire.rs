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

//! 1:1 translation of src/ModelingAlgorithms/TKTopAlgo/GTests/BRepLib_MakeWire_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkbrep::topods::Wire;
use rustycad_tktopalgo::brep_lib::BRepLibMakeWire;

#[test]
fn BRepLib_MakeWire_Test__OCC30708_2_InitializeWithNullWire() {
    let empty = Wire::default();

    // Should not throw exception when initializing with null wire
    // EXPECT_NO_THROW(BRepLib_MakeWire aWBuilder(empty));
    assert!(BRepLibMakeWire::from_wire(&empty).is_ok());
}
