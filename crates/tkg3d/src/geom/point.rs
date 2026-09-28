// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Point.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Point`: points of space as transient geometry.
use super::geometry::GeomGeometry;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkmath::gp::Point3;

/// A point with coordinates; `point.distance(&*other)`.
pub trait GeomPoint: GeomGeometry {
    /// Coordinates as `(x, y, z)` (`Coord`); `point.coord()`.
    fn coord(&self) -> (f64, f64, f64);
    /// A gp copy (`Pnt`); `point.pnt()`.
    fn pnt(&self) -> Point3;
    /// X coordinate (`X`); `point.x()`.
    fn x(&self) -> f64;
    /// Y coordinate (`Y`); `point.y()`.
    fn y(&self) -> f64;
    /// Z coordinate (`Z`); `point.z()`.
    fn z(&self) -> f64;
    /// Distance to `other` (`Distance`); `point.distance(&*other)`.
    fn distance(&self, other: &dyn GeomPoint) -> f64 {
        let first: Point3 = self.pnt();
        let second: Point3 = other.pnt();
        first.distance(&second)
    }
    /// Squared distance to `other` (`SquareDistance`); `point.square_distance(&*other)`.
    fn square_distance(&self, other: &dyn GeomPoint) -> f64 {
        let first: Point3 = self.pnt();
        let second: Point3 = other.pnt();
        first.square_distance(&second)
    }
}
impl_transient_interface!(dyn GeomPoint, "Geom_Point", dyn GeomGeometry);
