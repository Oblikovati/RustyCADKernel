// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! Behavior of the Intf port beyond the upstream suite (the bits are compared with
//! OCCT by the numeric probe).
use rustycad_tkgeomalgo::intf::{contain, plane_equation};
use rustycad_tkmath::gp::Point3;

#[test]
fn plane_equation_gives_the_unit_normal_and_the_polar_distance() {
    let (normal, distance) = plane_equation(
        Point3::new(0.0, 0.0, 2.0),
        Point3::new(3.0, 0.0, 2.0),
        Point3::new(0.0, 3.0, 2.0),
    );
    assert_eq!((normal.x(), normal.y(), normal.z()), (0.0, 0.0, 1.0));
    assert_eq!(distance, 2.0);
}

#[test]
fn plane_equation_of_a_degenerate_triangle_keeps_the_raw_normal() {
    // Native leaves the zero normal undivided and the polar distance at 0.
    let p = Point3::new(1.0, 2.0, 3.0);
    let (normal, distance) = plane_equation(p, p, Point3::new(4.0, 5.0, 6.0));
    assert_eq!((normal.modulus(), distance), (0.0, 0.0));
}

#[test]
fn contain_accepts_the_boundary_and_rejects_points_off_the_plane() {
    let (a, b, c) = (
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    );
    assert!(contain(a, b, c, Point3::new(0.5, 0.0, 0.0)));
    // Off the plane the edge cross products tilt apart, so the test fails.
    assert!(!contain(a, b, c, Point3::new(0.2, 0.2, 5.0)));
    assert!(!contain(a, b, c, Point3::new(1.0, 1.0, 0.0)));
}
