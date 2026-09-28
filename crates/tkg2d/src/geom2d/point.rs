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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Point.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Point`: points of the plane as transient geometry.
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name, dump_geometry};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::Point2;
use std::io;

/// A point with coordinates; `point.distance(&*other)`.
pub trait Geom2dPoint: Geom2dGeometry {
    /// Coordinates as `(x, y)` (`Coord`); `point.coord()`.
    fn coord(&self) -> (f64, f64);
    /// A gp copy (`Pnt2d`); `point.pnt2d()`.
    fn pnt2d(&self) -> Point2;
    /// X coordinate (`X`); `point.x()`.
    fn x(&self) -> f64;
    /// Y coordinate (`Y`); `point.y()`.
    fn y(&self) -> f64;
    /// Distance to `other` (`Distance`); `point.distance(&*other)`.
    fn distance(&self, other: &dyn Geom2dPoint) -> f64 {
        let first: Point2 = self.pnt2d();
        let second: Point2 = other.pnt2d();
        first.distance(second)
    }
    /// Squared distance to `other` (`SquareDistance`); `point.square_distance(&*other)`.
    fn square_distance(&self, other: &dyn Geom2dPoint) -> f64 {
        let first: Point2 = self.pnt2d();
        let second: Point2 = other.pnt2d();
        first.square_distance(second)
    }
}
impl_transient_interface!(dyn Geom2dPoint, "Geom2d_Point", dyn Geom2dGeometry);

/// `Geom2d_Point::DumpJson`; `dump_point(&mut stream, -1)?`.
pub fn dump_point(stream: &mut DumpStream, depth: i32) -> io::Result<()> {
    dump_class_name(stream, "Geom2d_Point")?;
    dump_base(stream, depth, dump_geometry)
}
