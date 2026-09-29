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

//! Direct translation of TopTrans_SurfaceTransition_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::toptrans::SurfaceTransition;
use rustycad_tkmath::gp::Direction3;

fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

#[test]
fn TopTrans_SurfaceTransition_Test__UndefinedStateStaysInstanceLocal() {
    let mut aReferenceTransition = SurfaceTransition::new();
    aReferenceTransition
        .reset_line(dir(1.0, 0.0, 0.0), dir(0.0, 0.0, 1.0))
        .unwrap();
    aReferenceTransition
        .compare_plane(
            precision::angular(),
            dir(0.0, 0.0, 1.0),
            Orientation::Forward,
            Orientation::Forward,
        )
        .unwrap();

    let aBefore: State = aReferenceTransition.state_before();
    let anAfter: State = aReferenceTransition.state_after();
    assert_ne!(aBefore, State::Unknown);
    assert_ne!(anAfter, State::Unknown);

    let mut anInvalidTransition = SurfaceTransition::new();
    anInvalidTransition
        .reset(
            dir(1.0, 0.0, 0.0),
            dir(0.0, 0.0, 1.0),
            dir(1.0, 0.0, 0.0),
            dir(1.0, 0.0, 0.0),
            1.0,
            1.0,
        )
        .unwrap();

    assert_eq!(anInvalidTransition.state_before(), State::Unknown);
    assert_eq!(anInvalidTransition.state_after(), State::Unknown);
    assert_eq!(aReferenceTransition.state_before(), aBefore);
    assert_eq!(aReferenceTransition.state_after(), anAfter);
}

#[test]
fn TopTrans_SurfaceTransition_Test__CompareUsesProvidedTolerance() {
    let mut aTransition = SurfaceTransition::new();
    aTransition
        .reset_line(dir(1.0, 0.0, 0.0), dir(0.0, 0.0, 1.0))
        .unwrap();
    aTransition
        .compare(
            1.0,
            dir(0.0, 0.0, 1.0),
            dir(0.6, 0.8, 0.0),
            dir(0.6, -0.8, 0.0),
            1.0,
            0.5,
            Orientation::Forward,
            Orientation::Forward,
        )
        .unwrap();

    assert_ne!(aTransition.state_before(), State::Unknown);
    assert_ne!(aTransition.state_after(), State::Unknown);
}
