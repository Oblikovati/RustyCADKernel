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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Intf at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Intf: interference (intersection points and tangent zones) between
//! polygons and polyhedra, and the triangle helpers of the `Intf` class.
use rustycad_tkmath::gp::{self, Point3, Xyz};

/// Intf::PlaneEquation: the unit normal of the triangle `p1 p2 p3` and its polar
/// distance (the normal dotted with `p1`).
///
/// The normal is the sum of the three edge cross products, as native sums them; for a
/// degenerate triangle (normal shorter than `gp::Resolution`) native leaves the
/// unnormalized normal and returns a polar distance of 0.
/// ```
/// use rustycad_tkgeomalgo::intf::plane_equation;
/// use rustycad_tkmath::gp::Point3;
/// let (normal, distance) = plane_equation(
///     Point3::new(0.0, 0.0, 1.0), Point3::new(1.0, 0.0, 1.0), Point3::new(0.0, 1.0, 1.0));
/// assert_eq!((normal.z(), distance), (1.0, 1.0));
/// ```
pub fn plane_equation(p1: Point3, p2: Point3, p3: Point3) -> (Xyz, f64) {
    let v1: Xyz = p2.xyz() - p1.xyz();
    let v2: Xyz = p3.xyz() - p2.xyz();
    let v3: Xyz = p1.xyz() - p3.xyz();
    let mut normal: Xyz = (v1 ^ v2) + (v2 ^ v3) + (v3 ^ v1);
    let length: f64 = normal.modulus();
    if length < gp::resolution() {
        return (normal, 0.0);
    }
    normal /= length;
    (normal, normal * p1.xyz())
}

/// Intf::Contain: whether `point` lies in the triangle `p1 p2 p3`, boundary included.
///
/// The three edge-to-point cross products must pairwise have non-negative dot products;
/// native uses no tolerance band, so a point exactly on an edge is inside.
/// ```
/// use rustycad_tkgeomalgo::intf::contain;
/// use rustycad_tkmath::gp::Point3;
/// let (a, b, c) = (Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, 0.0));
/// assert!(contain(a, b, c, Point3::new(0.25, 0.25, 0.0)));
/// ```
pub fn contain(p1: Point3, p2: Point3, p3: Point3, point: Point3) -> bool {
    let v1: Xyz = (p2.xyz() - p1.xyz()) ^ (point.xyz() - p1.xyz());
    let v2: Xyz = (p3.xyz() - p2.xyz()) ^ (point.xyz() - p2.xyz());
    let v3: Xyz = (p1.xyz() - p3.xyz()) ^ (point.xyz() - p3.xyz());
    v1 * v2 >= 0.0 && v2 * v3 >= 0.0 && v3 * v1 >= 0.0
}
