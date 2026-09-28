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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_CartesianPoint.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_CartesianPoint`: a point defined by its coordinates.
use super::cell::GeometryCell;
use super::geometry::{Geom2dGeometry, dump_base, dump_class_name, dump_field};
use super::point::{Geom2dPoint, dump_point};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Point2, Transform2};
use std::io;

/// A Cartesian point; `Handle::new(Geom2dCartesianPoint::new(1.0, 2.0))`.
#[derive(Default)]
pub struct Geom2dCartesianPoint {
    parent: StandardTransient,
    state: GeometryCell<Point2>,
}
impl_transient!(
    Geom2dCartesianPoint,
    "Geom2d_CartesianPoint",
    dyn Geom2dPoint,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dPoint]
);
impl Clone for Geom2dCartesianPoint {
    fn clone(&self) -> Self {
        Self::from_pnt2d(self.pnt2d())
    }
}
impl Geom2dCartesianPoint {
    /// Point from a gp point (`Geom2d_CartesianPoint(gp_Pnt2d)`);
    /// `Geom2dCartesianPoint::from_pnt2d(Point2::new(1.0, 2.0))`.
    pub fn from_pnt2d(point: Point2) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(point),
        }
    }
    /// Point from coordinates (`Geom2d_CartesianPoint(X, Y)`); `Geom2dCartesianPoint::new(1.0, 2.0)`.
    pub fn new(x: f64, y: f64) -> Self {
        Self::from_pnt2d(Point2::new(x, y))
    }
    /// Set both coordinates (`SetCoord`); `point.set_coord(1.0, 2.0)`.
    pub fn set_coord(&self, x: f64, y: f64) {
        self.state.write().set_coordinates(x, y);
    }
    /// Replace the point (`SetPnt2d`); `point.set_pnt2d(Point2::new(1.0, 2.0))`.
    pub fn set_pnt2d(&self, point: Point2) {
        *self.state.write() = point;
    }
    /// Set the X coordinate (`SetX`); `point.set_x(1.0)`.
    pub fn set_x(&self, x: f64) {
        self.state.write().set_x(x);
    }
    /// Set the Y coordinate (`SetY`); `point.set_y(2.0)`.
    pub fn set_y(&self, y: f64) {
        self.state.write().set_y(y);
    }
}
impl Geom2dGeometry for Geom2dCartesianPoint {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        self.state.write().transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let point: Point2 = self.pnt2d();
        dump_class_name(stream, "Geom2d_CartesianPoint")?;
        dump_base(stream, depth, dump_point)?;
        dump_field(stream, depth, "gpPnt2d", |nested, _| {
            point.dump_json(nested)
        })
    }
}
impl Geom2dPoint for Geom2dCartesianPoint {
    fn coord(&self) -> (f64, f64) {
        let point: Point2 = self.pnt2d();
        (point.x(), point.y())
    }
    fn pnt2d(&self) -> Point2 {
        *self.state.read()
    }
    fn x(&self) -> f64 {
        self.state.read().x()
    }
    fn y(&self) -> f64 {
        self.state.read().y()
    }
}
