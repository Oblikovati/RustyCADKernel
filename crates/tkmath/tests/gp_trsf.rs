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

//! OCC23361: composed and sequential transforms must produce equal points.
#![allow(non_snake_case, clippy::excessive_precision)]
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3};
#[test]
fn gp_TrsfTest__OCC23361_TransformationComposition() {
    let origin = Point3::new(0.0, 0.0, 2.0);
    let first =
        Transform3::rotation(Axis1::new(origin, Direction3::Y), -0.49328285294022267).unwrap();
    let second =
        Transform3::rotation(Axis1::new(origin, Direction3::Z), 0.87538474718473880).unwrap();
    let composed = second.multiplied(first);
    let point = Point3::new(10.0, 3.0, 4.0);
    let combined_point = point.transformed(composed);
    let mut sequential_point = point.transformed(first);
    sequential_point.transform(second);
    assert!(combined_point.is_equal(&sequential_point, 1e-7));
}
